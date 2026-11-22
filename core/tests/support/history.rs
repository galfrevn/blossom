use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{Value, json};
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

use crate::answers::{answer_discovery_naming, answer_me};

pub const THREADS_PER_PAGE: usize = 2;
pub const EVENTS_PER_PAGE: usize = 3;
const REPLIES_PER_PAGE: usize = 500;
pub const FRAMES_PER_STREAM: usize = 2;
const PAUSE_WHEN_CAUGHT_UP: Duration = Duration::from_millis(20);
pub const SLOW_PAGE: Duration = Duration::from_millis(2500);
const STALLED_STREAM: Duration = Duration::from_secs(10);
const LEASE_MINUTES: u32 = 5;

pub fn time_at(minute: u32) -> String {
    let hour = 10 + minute / 60;
    let minute_of_the_hour = minute % 60;
    format!("2026-11-20T{hour:02}:{minute_of_the_hour:02}:00.000Z")
}

pub fn lease_until(claimed_at_minute: u32) -> String {
    time_at(claimed_at_minute + LEASE_MINUTES)
}

#[derive(Clone)]
pub struct FakeActor {
    pub id: String,
    principal: String,
    profile: Option<String>,
    kind: String,
    handle: String,
    pub name: String,
    via: Option<String>,
}

impl FakeActor {
    pub fn person(handle: &str, name: &str) -> FakeActor {
        FakeActor {
            id: format!("local:user:{handle}"),
            principal: format!("local:user:{handle}"),
            profile: None,
            kind: String::from("person"),
            handle: handle.to_owned(),
            name: name.to_owned(),
            via: None,
        }
    }

    pub fn agent(profile: &str, person: &str) -> FakeActor {
        FakeActor {
            id: format!("local:user:{person}+{profile}"),
            principal: format!("local:user:{person}"),
            profile: Some(profile.to_owned()),
            kind: String::from("agent"),
            handle: format!("{person}/{profile}"),
            name: profile.to_owned(),
            via: Some(person.to_owned()),
        }
    }

    pub fn principal(&self) -> &str {
        &self.principal
    }

    fn author(&self) -> Value {
        let mut author = json!({
            "kind": self.kind,
            "handle": self.handle,
            "name": self.name,
            "avatar": Value::Null
        });
        if let Some(via) = &self.via {
            author["via"] = json!({ "name": via, "environment": Value::Null });
        }
        author
    }

    fn listed(&self) -> Value {
        let mut listed = self.author();
        listed["id"] = Value::from(self.id.as_str());
        listed
    }

    fn by(&self) -> Value {
        json!({ "actor": self.id, "principal": self.principal, "profile": self.profile })
    }
}

fn the_person_with_the_token() -> FakeActor {
    FakeActor::person("ada", "Ada Lovelace")
}

#[derive(Clone)]
struct DeliveredEvent {
    event_type: String,
    created_at: String,
    payload: Value,
    actor: Option<FakeActor>,
    message: Value,
}

#[derive(Clone)]
enum HistoryEvent {
    Delivered(Box<DeliveredEvent>),
    Bookkeeping,
    Unreadable,
}

impl HistoryEvent {
    fn message(&self) -> Option<&Value> {
        match self {
            HistoryEvent::Delivered(delivered) => Some(&delivered.message),
            HistoryEvent::Bookkeeping | HistoryEvent::Unreadable => None,
        }
    }
}

#[derive(Clone, Default)]
pub struct BoardHistory {
    events: Vec<HistoryEvent>,
    actors_known_without_an_event: Vec<FakeActor>,
}

impl BoardHistory {
    pub fn post(&mut self, id: &str, channel: &str, thread_root: &str, minute: u32, text: &str) {
        self.post_as(
            &the_person_with_the_token(),
            id,
            channel,
            thread_root,
            minute,
            text,
        );
    }

    pub fn post_as(
        &mut self,
        author: &FakeActor,
        id: &str,
        channel: &str,
        thread_root: &str,
        minute: u32,
        text: &str,
    ) {
        let thread = format!("thread-{thread_root}");
        let re = if id == thread_root {
            Value::Null
        } else {
            Value::from(thread_root)
        };
        let message = json!({
            "id": id,
            "channel": channel,
            "thread": thread,
            "re": re,
            "createdAt": time_at(minute),
            "version": self.next_version(),
            "text": text,
            "by": author.by(),
            "author": author.author()
        });
        let payload = json!({
            "channel": channel,
            "text": text,
            "re": re,
            "thread": thread,
            "by": author.by()
        });
        self.deliver(DeliveredEvent {
            event_type: String::from("message.posted"),
            created_at: time_at(minute),
            payload,
            actor: Some(author.clone()),
            message,
        });
    }

    pub fn edit(&mut self, id: &str, text: &str) {
        let author = the_person_with_the_token();
        let mut message = self.current_view(id);
        message["version"] = Value::from(self.next_version());
        message["text"] = Value::from(text);
        let created_at = message["createdAt"].as_str().unwrap_or_default().to_owned();
        let payload = json!({ "id": id, "text": text, "by": author.by(), "from": Value::Null });
        self.deliver(DeliveredEvent {
            event_type: String::from("message.edited"),
            created_at,
            payload,
            actor: Some(author),
            message,
        });
    }

    fn deliver(&mut self, delivered: DeliveredEvent) {
        self.events
            .push(HistoryEvent::Delivered(Box::new(delivered)));
    }

    pub fn bookkeeping(&mut self) {
        self.events.push(HistoryEvent::Bookkeeping);
    }

    pub fn unreadable_event(&mut self) {
        self.events.push(HistoryEvent::Unreadable);
    }

    pub fn know_actor(&mut self, actor: FakeActor) {
        self.actors_known_without_an_event.push(actor);
    }

    pub fn create_task(
        &mut self,
        id: &str,
        by: &FakeActor,
        minute: u32,
        assigned_to: Option<&str>,
        needs: &[&str],
    ) {
        let state = if needs.is_empty() { "ready" } else { "waiting" };
        let task = json!({
            "state": state,
            "owner": by.principal,
            "by": by.id,
            "for": assigned_to,
            "needs": needs,
            "epoch": 0,
            "claim": Value::Null,
            "blocked": Value::Null,
            "result": Value::Null,
            "reason": Value::Null
        });
        let payload = json!({ "needs": needs, "for": assigned_to });
        self.change_task(id, "task.created", minute, Some(by), payload, |root_task| {
            *root_task = task;
        });
    }

    pub fn claim_task(&mut self, id: &str, by: &FakeActor, minute: u32) {
        let epoch = self.task_epoch(id) + 1;
        let until = lease_until(minute);
        let claim = json!({ "actor": by.id, "until": until });
        let payload = json!({ "epoch": epoch, "until": until });
        self.change_task(id, "task.claimed", minute, Some(by), payload, |task| {
            task["state"] = Value::from("claimed");
            task["epoch"] = Value::from(epoch);
            task["claim"] = claim;
        });
    }

    pub fn release_task(&mut self, id: &str, by: &FakeActor, minute: u32) {
        let payload = json!({ "epoch": self.task_epoch(id) });
        self.change_task(id, "task.released", minute, Some(by), payload, |task| {
            task["state"] = Value::from("ready");
            task["claim"] = Value::Null;
        });
    }

    pub fn expire_task(&mut self, id: &str, minute: u32) {
        let payload = json!({ "epoch": self.task_epoch(id) });
        self.change_task(id, "task.expired", minute, None, payload, |task| {
            task["state"] = Value::from("ready");
            task["claim"] = Value::Null;
        });
    }

    pub fn complete_task(&mut self, id: &str, by: &FakeActor, minute: u32, result: &str) {
        let payload = json!({ "epoch": self.task_epoch(id), "result": result });
        self.change_task(id, "task.completed", minute, Some(by), payload, |task| {
            task["state"] = Value::from("completed");
            task["claim"] = Value::Null;
            task["result"] = Value::from(result);
        });
    }

    pub fn fail_task(&mut self, id: &str, by: &FakeActor, minute: u32, reason: &str) {
        let payload = json!({ "epoch": self.task_epoch(id), "reason": reason });
        self.change_task(id, "task.failed", minute, Some(by), payload, |task| {
            task["state"] = Value::from("failed");
            task["claim"] = Value::Null;
            task["reason"] = Value::from(reason);
        });
    }

    pub fn block_task(&mut self, id: &str, by: &FakeActor, minute: u32, kind: &str, reason: &str) {
        let payload = json!({ "kind": kind, "reason": reason });
        self.change_task(id, "task.blocked", minute, Some(by), payload, |task| {
            task["state"] = Value::from("blocked");
            task["claim"] = Value::Null;
            task["blocked"] = json!({ "kind": kind, "reason": reason });
        });
    }

    pub fn unblock_task(&mut self, id: &str, by: &FakeActor, minute: u32) {
        self.change_task(id, "task.unblocked", minute, Some(by), json!({}), |task| {
            task["state"] = Value::from("ready");
            task["blocked"] = Value::Null;
        });
    }

    pub fn reopen_task(&mut self, id: &str, by: &FakeActor, minute: u32) {
        self.change_task(id, "task.reopened", minute, Some(by), json!({}), |task| {
            task["state"] = Value::from("ready");
            task["reason"] = Value::Null;
        });
    }

    pub fn cancel_task(&mut self, id: &str, by: &FakeActor, minute: u32, reason: &str) {
        let payload = json!({ "reason": reason });
        self.change_task(id, "task.canceled", minute, Some(by), payload, |task| {
            task["state"] = Value::from("canceled");
            task["claim"] = Value::Null;
            task["reason"] = Value::from(reason);
        });
    }

    fn change_task(
        &mut self,
        id: &str,
        event_type: &str,
        minute: u32,
        by: Option<&FakeActor>,
        mut payload: Value,
        change: impl FnOnce(&mut Value),
    ) {
        let mut root = self.current_view(id);
        root["version"] = Value::from(self.next_version());
        change(&mut root["task"]);
        payload["id"] = Value::from(id);
        payload["thread"] = root["thread"].clone();
        if let Some(by) = by {
            payload["by"] = by.by();
        }
        self.deliver(DeliveredEvent {
            event_type: event_type.to_owned(),
            created_at: time_at(minute),
            payload,
            actor: by.cloned(),
            message: root,
        });
    }

    fn task_epoch(&self, id: &str) -> u64 {
        self.current_view(id)["task"]["epoch"]
            .as_u64()
            .unwrap_or_default()
    }

    fn next_version(&self) -> u64 {
        self.events.len() as u64 + 1
    }

    pub fn current_view(&self, id: &str) -> Value {
        self.events
            .iter()
            .rev()
            .filter_map(HistoryEvent::message)
            .find(|message| message["id"] == id)
            .cloned()
            .unwrap_or(Value::Null)
    }

    fn current_views(&self) -> BTreeMap<String, Value> {
        let mut views = BTreeMap::new();
        for message in self.events.iter().filter_map(HistoryEvent::message) {
            let id = message["id"].as_str().unwrap_or_default().to_owned();
            views.insert(id, message.clone());
        }
        views
    }

    fn listed_actors(&self) -> BTreeMap<String, FakeActor> {
        let mut listed = BTreeMap::new();
        let actors_who_acted = self.events.iter().filter_map(|event| match event {
            HistoryEvent::Delivered(delivered) => delivered.actor.as_ref(),
            HistoryEvent::Bookkeeping | HistoryEvent::Unreadable => None,
        });
        for actor in actors_who_acted {
            listed.insert(actor.id.clone(), actor.clone());
        }
        for actor in &self.actors_known_without_an_event {
            listed.insert(actor.id.clone(), actor.clone());
        }
        listed
    }

    fn cursor_of(position: usize) -> String {
        format!("event-{position}")
    }

    fn latest_cursor(&self) -> String {
        Self::cursor_of(self.events.len().saturating_sub(1))
    }

    fn position_after(&self, cursor: &str) -> Option<usize> {
        if let Some(position) = cursor.strip_prefix("event-") {
            return position.parse::<usize>().ok().map(|position| position + 1);
        }
        let posted_at = self.events.iter().position(|event| {
            event
                .message()
                .is_some_and(|message| message["id"] == cursor && message["re"].is_null())
        });
        posted_at.map(|position| position + 1)
    }

    fn served_event(position: usize, delivered: &DeliveredEvent) -> Value {
        let mut served = json!({
            "cursor": Self::cursor_of(position),
            "type": delivered.event_type,
            "createdAt": delivered.created_at,
            "payload": delivered.payload,
            "message": delivered.message
        });
        if let Some(actor) = &delivered.actor {
            served["actor"] = actor.listed();
        }
        served
    }

    fn replies_to(views: &BTreeMap<String, Value>, root_id: &Value) -> Vec<Value> {
        let mut replies: Vec<Value> = views
            .values()
            .filter(|view| view["re"] == *root_id)
            .cloned()
            .collect();
        replies.sort_by(|first, second| {
            first["createdAt"]
                .as_str()
                .cmp(&second["createdAt"].as_str())
        });
        replies
    }

    fn latest_activity(root: &Value, replies: &[Value]) -> String {
        replies
            .iter()
            .chain(std::iter::once(root))
            .filter_map(|view| view["createdAt"].as_str())
            .max()
            .unwrap_or_default()
            .to_owned()
    }

    fn thread_around(&self, id: &str, limit: Option<usize>) -> Option<Value> {
        let views = self.current_views();
        let message = views.get(id)?;
        let root = match message["re"].as_str() {
            Some(root_id) => views.get(root_id)?,
            None => message,
        };
        let replies = Self::replies_to(&views, &root["id"]);
        let latest = replies
            .len()
            .saturating_sub(limit.unwrap_or(REPLIES_PER_PAGE));
        let cursor = self.latest_cursor();
        Some(json!({
            "message": root,
            "replies": replies[latest..],
            "cursor": cursor,
            "schedules": []
        }))
    }

    fn threads_page(&self, before: Option<&str>) -> Value {
        let views = self.current_views();
        let mut threads: Vec<(String, Value)> = views
            .values()
            .filter(|view| view["re"].is_null())
            .map(|root| {
                let replies = Self::replies_to(&views, &root["id"]);
                let activity = Self::latest_activity(root, &replies);
                let last = replies.last().cloned().unwrap_or(Value::Null);
                let summary = json!({ "message": root, "replies": replies.len(), "last": last });
                (activity, summary)
            })
            .collect();
        threads.sort_by(|first, second| first.0.cmp(&second.0));
        let end = before
            .and_then(|token| token.strip_prefix("older-"))
            .and_then(|index| index.parse::<usize>().ok())
            .unwrap_or(threads.len());
        let start = end.saturating_sub(THREADS_PER_PAGE);
        let page: Vec<Value> = threads[start..end]
            .iter()
            .map(|(_, summary)| summary.clone())
            .collect();
        let older = if start == 0 {
            Value::Null
        } else {
            Value::from(format!("older-{start}"))
        };
        json!({
            "threads": page,
            "cursor": self.latest_cursor(),
            "older": older
        })
    }

    fn channels_page(&self) -> Value {
        let mut counts: BTreeMap<String, u64> = BTreeMap::new();
        for view in self.current_views().values() {
            let channel = view["channel"].as_str().unwrap_or_default().to_owned();
            *counts.entry(channel).or_default() += 1;
        }
        let channels: Vec<Value> = counts
            .iter()
            .map(|(path, messages)| json!({ "path": path, "messages": messages }))
            .collect();
        json!({ "channels": channels, "cursor": "channels", "next": Value::Null })
    }

    fn actors_page(&self, asked_ids: &[String]) -> Value {
        let listed = self.listed_actors();
        let mut answered: Vec<&FakeActor> = listed
            .values()
            .filter(|actor| asked_ids.contains(&actor.id))
            .collect();
        answered.sort_by(|first, second| first.handle.cmp(&second.handle));
        let actors: Vec<Value> = answered.iter().map(|actor| actor.listed()).collect();
        json!({ "actors": actors, "cursor": self.latest_cursor() })
    }

    fn events_stream(&self, after: &str) -> Option<(String, Vec<usize>)> {
        let first = self.position_after(after)?;
        let last = (first + FRAMES_PER_STREAM).min(self.events.len());
        let mut body = format!(": connected\n\nid: {after}\n\n");
        for (position, event) in self.events.iter().enumerate().take(last).skip(first) {
            let cursor = Self::cursor_of(position);
            match event {
                HistoryEvent::Delivered(delivered) => {
                    let data = Self::served_event(position, delivered);
                    body.push_str(&format!("id: {cursor}\nevent: board\ndata: {data}\n\n"));
                }
                HistoryEvent::Bookkeeping => {
                    body.push_str(&format!("id: {cursor}\n\n"));
                }
                HistoryEvent::Unreadable => {
                    body.push_str(&format!("id: {cursor}\nevent: board\ndata: not json\n\n"));
                }
            }
        }
        Some((body, (first..last).collect()))
    }

    fn events_page(&self, after: &str) -> Option<Value> {
        let first = self.position_after(after)?;
        let last = (first + EVENTS_PER_PAGE).min(self.events.len());
        let events: Vec<Value> = (first..last)
            .filter_map(|position| match &self.events[position] {
                HistoryEvent::Delivered(delivered) => Some(Self::served_event(position, delivered)),
                HistoryEvent::Bookkeeping | HistoryEvent::Unreadable => None,
            })
            .collect();
        let cursor = if last > first {
            Self::cursor_of(last - 1)
        } else {
            after.to_owned()
        };
        Some(json!({ "events": events, "cursor": cursor }))
    }
}

#[derive(Clone, Default)]
pub struct FakeBoard {
    pub history: Arc<Mutex<BoardHistory>>,
    pub pages_to_refuse: Arc<AtomicUsize>,
    pub pages_to_slow_down: Arc<AtomicUsize>,
    pub streams_to_refuse: Arc<AtomicUsize>,
    pub streams_to_stall: Arc<AtomicUsize>,
    pub streams_end_at_once: Arc<AtomicBool>,
    pub streams_refuse_every_cursor: Arc<AtomicBool>,
    pub stream_starts: Arc<Mutex<Vec<String>>>,
    pub streamed_positions: Arc<Mutex<Vec<usize>>>,
}

impl FakeBoard {
    pub fn with_history(history: BoardHistory) -> FakeBoard {
        FakeBoard {
            history: Arc::new(Mutex::new(history)),
            ..FakeBoard::default()
        }
    }

    pub fn change_history(&self, change: impl FnOnce(&mut BoardHistory)) {
        let mut history = self
            .history
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        change(&mut history);
    }
}

struct HistoryResponder {
    board: FakeBoard,
}

impl HistoryResponder {
    fn respond_with_stream(&self, request: &Request) -> ResponseTemplate {
        if take_one(&self.board.streams_to_refuse) {
            return ResponseTemplate::new(503);
        }
        if self
            .board
            .streams_refuse_every_cursor
            .load(Ordering::SeqCst)
        {
            return refused_cursor();
        }
        if take_one(&self.board.streams_to_stall) {
            return ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_delay(STALLED_STREAM);
        }
        let Some(after) = request
            .headers
            .get("last-event-id")
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned)
        else {
            return ResponseTemplate::new(400);
        };
        self.board
            .stream_starts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(after.clone());
        if self.board.streams_end_at_once.load(Ordering::SeqCst) {
            return ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(": connected\n\n");
        }
        let history = self
            .board
            .history
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some((body, positions)) = history.events_stream(&after) else {
            return refused_cursor();
        };
        let caught_up = positions.is_empty();
        self.board
            .streamed_positions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .extend(positions);
        let response = ResponseTemplate::new(200)
            .insert_header("content-type", "text/event-stream")
            .set_body_string(body);
        if caught_up {
            return response.set_delay(PAUSE_WHEN_CAUGHT_UP);
        }
        response
    }
}

impl Respond for HistoryResponder {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let wants_a_stream = request
            .headers
            .get("accept")
            .is_some_and(|value| value.as_bytes() == b"text/event-stream");
        if request.url.path() == "/api/board/events" && wants_a_stream {
            return self.respond_with_stream(request);
        }
        if take_one(&self.board.pages_to_refuse) {
            return ResponseTemplate::new(503);
        }
        let query: BTreeMap<String, String> = request
            .url
            .query_pairs()
            .map(|(key, value)| (key.into_owned(), value.into_owned()))
            .collect();
        let asked_ids = asked_actor_ids(request);
        let history = self
            .board
            .history
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let path = request.url.path();
        let body = match path {
            "/api/board/threads" => history.threads_page(query.get("before").map(String::as_str)),
            "/api/board/channels" => history.channels_page(),
            "/api/board/actors" => history.actors_page(&asked_ids),
            "/api/board/events" => match query.get("after").map(|after| history.events_page(after))
            {
                Some(Some(page)) => page,
                Some(None) => return refused_cursor(),
                None => return ResponseTemplate::new(400),
            },
            _ => match path.strip_prefix("/api/board/messages/") {
                Some(id) => match history.thread_around(id, query_number(&query, "limit")) {
                    Some(thread) => thread,
                    None => return message_not_found(),
                },
                None => return ResponseTemplate::new(404),
            },
        };
        let response = ResponseTemplate::new(200).set_body_json(body);
        if take_one(&self.board.pages_to_slow_down) {
            return response.set_delay(SLOW_PAGE);
        }
        response
    }
}

pub fn asked_actor_ids(request: &Request) -> Vec<String> {
    request
        .url
        .query_pairs()
        .filter(|(key, _)| key == "id")
        .map(|(_, value)| value.into_owned())
        .collect()
}

fn query_number(query: &BTreeMap<String, String>, key: &str) -> Option<usize> {
    query.get(key).and_then(|value| value.parse().ok())
}

fn refused_cursor() -> ResponseTemplate {
    let error = json!({ "error": { "code": "invalid_cursor", "message": "not a cursor here" } });
    ResponseTemplate::new(400).set_body_json(error)
}

fn message_not_found() -> ResponseTemplate {
    let error = json!({ "error": { "code": "message_not_found", "message": "message not found" } });
    ResponseTemplate::new(404).set_body_json(error)
}

fn take_one(remaining: &AtomicUsize) -> bool {
    remaining
        .try_update(Ordering::SeqCst, Ordering::SeqCst, |count| {
            count.checked_sub(1)
        })
        .is_ok()
}

pub async fn serve_board(server: &MockServer, board: FakeBoard) {
    let url = format!("{}/api/board", server.uri());
    answer_discovery_naming(server, &url).await;
    answer_me(server, "token-of-ada", "ada", "Ada Lovelace").await;
    Mock::given(method("GET"))
        .and(path_regex(
            "^/api/board/(threads|channels|events|actors|messages/[^/]+)$",
        ))
        .respond_with(HistoryResponder { board })
        .mount(server)
        .await;
}

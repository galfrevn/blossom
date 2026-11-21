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
pub const FRAMES_PER_STREAM: usize = 2;
const PAUSE_WHEN_CAUGHT_UP: Duration = Duration::from_millis(20);
pub const SLOW_PAGE: Duration = Duration::from_millis(2500);
const STALLED_STREAM: Duration = Duration::from_secs(10);

#[derive(Clone)]
pub struct HistoryEvent {
    pub message: Value,
    pub carries_a_message: bool,
    pub unreadable: bool,
}

#[derive(Clone, Default)]
pub struct BoardHistory {
    pub events: Vec<HistoryEvent>,
}

impl BoardHistory {
    pub fn post(&mut self, id: &str, channel: &str, thread_root: &str, minute: u32, text: &str) {
        let version = self.events.len() as u64 + 1;
        self.events.push(HistoryEvent {
            message: json!({
                "id": id,
                "channel": channel,
                "thread": format!("thread-{thread_root}"),
                "re": if id == thread_root { Value::Null } else { Value::from(thread_root) },
                "createdAt": format!("2026-11-20T10:{minute:02}:00.000Z"),
                "version": version,
                "text": text
            }),
            carries_a_message: true,
            unreadable: false,
        });
    }

    pub fn edit(&mut self, id: &str, text: &str) {
        let version = self.events.len() as u64 + 1;
        let mut message = self.current_view(id);
        message["version"] = Value::from(version);
        message["text"] = Value::from(text);
        self.events.push(HistoryEvent {
            message,
            carries_a_message: true,
            unreadable: false,
        });
    }

    pub fn bookkeeping(&mut self) {
        self.events.push(HistoryEvent {
            message: Value::Null,
            carries_a_message: false,
            unreadable: false,
        });
    }

    pub fn unreadable_event(&mut self) {
        self.events.push(HistoryEvent {
            message: Value::Null,
            carries_a_message: false,
            unreadable: true,
        });
    }

    pub fn current_view(&self, id: &str) -> Value {
        self.events
            .iter()
            .rev()
            .find(|event| event.carries_a_message && event.message["id"] == id)
            .map(|event| event.message.clone())
            .unwrap_or(Value::Null)
    }

    fn current_views(&self) -> BTreeMap<String, Value> {
        let mut views = BTreeMap::new();
        for event in self.events.iter().filter(|event| event.carries_a_message) {
            let id = event.message["id"].as_str().unwrap_or_default().to_owned();
            views.insert(id, event.message.clone());
        }
        views
    }

    fn cursor_of(position: usize) -> String {
        format!("event-{position}")
    }

    fn position_after(&self, cursor: &str) -> Option<usize> {
        if let Some(position) = cursor.strip_prefix("event-") {
            return position.parse::<usize>().ok().map(|position| position + 1);
        }
        let posted_at = self.events.iter().position(|event| {
            event.carries_a_message
                && event.message["id"] == cursor
                && event.message["re"].is_null()
        });
        posted_at.map(|position| position + 1)
    }

    fn thread_around(&self, id: &str) -> Option<Value> {
        let message = self.current_view(id);
        if message.is_null() {
            return None;
        }
        let cursor = Self::cursor_of(self.events.len().saturating_sub(1));
        Some(json!({ "message": message, "replies": [], "cursor": cursor, "schedules": [] }))
    }

    fn threads_page(&self, before: Option<&str>) -> Value {
        let views = self.current_views();
        let mut roots: Vec<&Value> = views.values().filter(|view| view["re"].is_null()).collect();
        roots.sort_by(|first, second| {
            first["createdAt"]
                .as_str()
                .cmp(&second["createdAt"].as_str())
        });
        let end = before
            .and_then(|token| token.strip_prefix("older-"))
            .and_then(|index| index.parse::<usize>().ok())
            .unwrap_or(roots.len());
        let start = end.saturating_sub(THREADS_PER_PAGE);
        let threads: Vec<Value> = roots[start..end]
            .iter()
            .map(|root| {
                let replies = views
                    .values()
                    .filter(|view| view["re"] == root["id"])
                    .count();
                json!({ "message": root, "replies": replies, "last": Value::Null })
            })
            .collect();
        let older = if start == 0 {
            Value::Null
        } else {
            Value::from(format!("older-{start}"))
        };
        json!({
            "threads": threads,
            "cursor": Self::cursor_of(self.events.len().saturating_sub(1)),
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

    fn events_stream(&self, after: &str) -> Option<(String, Vec<usize>)> {
        let first = self.position_after(after)?;
        let last = (first + FRAMES_PER_STREAM).min(self.events.len());
        let mut body = format!(": connected\n\nid: {after}\n\n");
        for (position, event) in self.events.iter().enumerate().take(last).skip(first) {
            let cursor = Self::cursor_of(position);
            if event.unreadable {
                body.push_str(&format!("id: {cursor}\nevent: board\ndata: not json\n\n"));
            } else if event.carries_a_message {
                let data = json!({
                    "cursor": cursor,
                    "type": "message.posted",
                    "message": event.message
                });
                body.push_str(&format!("id: {cursor}\nevent: board\ndata: {data}\n\n"));
            } else {
                body.push_str(&format!("id: {cursor}\n\n"));
            }
        }
        Some((body, (first..last).collect()))
    }

    fn events_page(&self, after: &str) -> Option<Value> {
        let first = self.position_after(after)?;
        let last = (first + EVENTS_PER_PAGE).min(self.events.len());
        let events: Vec<Value> = (first..last)
            .filter(|position| self.events[*position].carries_a_message)
            .map(|position| {
                json!({
                    "cursor": Self::cursor_of(position),
                    "type": "message.posted",
                    "message": self.events[position].message
                })
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
        let history = self
            .board
            .history
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let path = request.url.path();
        let body = match path {
            "/api/board/threads" => history.threads_page(query.get("before").map(String::as_str)),
            "/api/board/channels" => history.channels_page(),
            "/api/board/events" => match query.get("after").map(|after| history.events_page(after))
            {
                Some(Some(page)) => page,
                Some(None) => return refused_cursor(),
                None => return ResponseTemplate::new(400),
            },
            _ => match path.strip_prefix("/api/board/messages/") {
                Some(id) => match history.thread_around(id) {
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
            "^/api/board/(threads|channels|events|messages/[^/]+)$",
        ))
        .respond_with(HistoryResponder { board })
        .mount(server)
        .await;
}

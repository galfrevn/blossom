use std::collections::BTreeMap;

use serde_json::{Value, json};
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

pub const THREADS_PER_PAGE: usize = 2;
pub const EVENTS_PER_PAGE: usize = 3;

#[derive(Clone)]
pub struct HistoryEvent {
    pub message: Value,
    pub carries_a_message: bool,
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
        });
    }

    pub fn bookkeeping(&mut self) {
        self.events.push(HistoryEvent {
            message: Value::Null,
            carries_a_message: false,
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

    fn position_after(&self, cursor: &str) -> usize {
        if let Some(position) = cursor.strip_prefix("event-") {
            return position.parse::<usize>().unwrap_or_default() + 1;
        }
        let posted_at = self.events.iter().position(|event| {
            event.carries_a_message
                && event.message["id"] == cursor
                && event.message["re"].is_null()
        });
        posted_at.map_or(0, |position| position + 1)
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

    fn events_page(&self, after: &str) -> Value {
        let first = self.position_after(after);
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
        json!({ "events": events, "cursor": cursor })
    }
}

struct HistoryResponder {
    history: BoardHistory,
}

impl Respond for HistoryResponder {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let query: BTreeMap<String, String> = request
            .url
            .query_pairs()
            .map(|(key, value)| (key.into_owned(), value.into_owned()))
            .collect();
        let body = match request.url.path() {
            "/api/board/threads" => self
                .history
                .threads_page(query.get("before").map(String::as_str)),
            "/api/board/channels" => self.history.channels_page(),
            "/api/board/events" => match query.get("after") {
                Some(after) => self.history.events_page(after),
                None => return ResponseTemplate::new(400),
            },
            _ => return ResponseTemplate::new(404),
        };
        ResponseTemplate::new(200).set_body_json(body)
    }
}

pub async fn serve_history(server: &MockServer, history: BoardHistory) {
    Mock::given(method("GET"))
        .and(path_regex("^/api/board/(threads|channels|events)$"))
        .respond_with(HistoryResponder { history })
        .mount(server)
        .await;
}

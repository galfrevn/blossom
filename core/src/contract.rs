mod tasks;

use serde::Serialize;
use ts_rs::{Config, TS};

pub use tasks::{Lease, TaskChange, TaskCount, TaskNarrowing, TaskRow, TaskWindow};

pub fn typescript() -> String {
    let configuration = Config::new();
    let declarations = [
        Person::decl(&configuration),
        OpenBoard::decl(&configuration),
        StopReason::decl(&configuration),
        ReplicaState::decl(&configuration),
        MentionedActor::decl(&configuration),
        FlowRow::decl(&configuration),
        FlowFold::decl(&configuration),
        FlowWindow::decl(&configuration),
        FlowChange::decl(&configuration),
        ThreadMessage::decl(&configuration),
        Thread::decl(&configuration),
        HeldTask::decl(&configuration),
        Agent::decl(&configuration),
        TextPart::decl(&configuration),
        FoundMessage::decl(&configuration),
        FoundTask::decl(&configuration),
        FoundChannel::decl(&configuration),
        FoundAgent::decl(&configuration),
        SearchResults::decl(&configuration),
        TaskNarrowing::decl(&configuration),
        TaskRow::decl(&configuration),
        Lease::decl(&configuration),
        TaskWindow::decl(&configuration),
        TaskCount::decl(&configuration),
        TaskChange::decl(&configuration),
    ];
    declarations
        .iter()
        .map(|declaration| format!("export {declaration}\n"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct Person {
    pub handle: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum OpenBoard {
    NoneRunning,
    Open { url: String, person: Person },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ReplicaState {
    Copying { stored: u32, total: Option<u32> },
    Live { stored: u32 },
    Reconnecting { stored: u32 },
    Stopped { stored: u32, reason: StopReason },
}

impl ReplicaState {
    pub fn stored(&self) -> u32 {
        match self {
            ReplicaState::Copying { stored, .. }
            | ReplicaState::Live { stored }
            | ReplicaState::Reconnecting { stored }
            | ReplicaState::Stopped { stored, .. } => *stored,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum StopReason {
    UnreadableBoard,
    UnwritableReplica,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct MentionedActor {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct FlowRow {
    pub id: String,
    pub channel: String,
    pub author: String,
    pub on_behalf_of: Option<String>,
    pub created_at: String,
    pub text: String,
    pub mentions: Vec<MentionedActor>,
    pub edited: bool,
    pub deleted: bool,
    pub first_of_its_minute: bool,
    pub fold: Option<FlowFold>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct FlowFold {
    pub count: u32,
    pub agents: u32,
    pub recent: Vec<FlowRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct FlowWindow {
    pub total: u32,
    pub start: u32,
    pub rows: Vec<FlowRow>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
pub struct FlowChange {
    pub revision: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ThreadMessage {
    pub id: String,
    pub author: String,
    pub on_behalf_of: Option<String>,
    pub created_at: String,
    pub text: String,
    pub mentions: Vec<MentionedActor>,
    pub edited: bool,
    pub deleted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct Thread {
    pub id: String,
    pub channel: String,
    pub messages: Vec<ThreadMessage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct HeldTask {
    pub task: String,
    pub until: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct TextPart {
    pub text: String,
    pub marked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct FoundMessage {
    pub id: String,
    pub channel: String,
    pub author: String,
    pub created_at: String,
    pub snippet: Vec<TextPart>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct FoundTask {
    pub id: String,
    pub channel: String,
    pub state: String,
    pub snippet: Vec<TextPart>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub on_behalf_of: Option<String>,
    pub holds: Option<HeldTask>,
    pub lapsed: bool,
    pub failed_recently: bool,
    pub channel: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct FoundChannel {
    pub path: String,
    pub marked_path: Vec<TextPart>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct FoundAgent {
    pub id: String,
    pub marked_name: Vec<TextPart>,
    pub marked_handle: Vec<TextPart>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, TS)]
pub struct SearchResults {
    pub messages: Vec<FoundMessage>,
    pub tasks: Vec<FoundTask>,
    pub channels: Vec<FoundChannel>,
    pub agents: Vec<FoundAgent>,
}

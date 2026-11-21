use serde::Serialize;
use ts_rs::{Config, TS};

pub fn typescript() -> String {
    let configuration = Config::new();
    let declarations = [
        Person::decl(&configuration),
        OpenBoard::decl(&configuration),
        StopReason::decl(&configuration),
        ReplicaState::decl(&configuration),
        FlowRow::decl(&configuration),
        FlowFold::decl(&configuration),
        FlowWindow::decl(&configuration),
        FlowChange::decl(&configuration),
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
#[serde(rename_all = "camelCase")]
pub struct FlowRow {
    pub id: String,
    pub channel: String,
    pub author: String,
    pub on_behalf_of: Option<String>,
    pub created_at: String,
    pub text: String,
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

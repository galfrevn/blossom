use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TaskNarrowing {
    pub channel: Option<String>,
    pub state: Option<String>,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TaskRow {
    pub id: String,
    pub state: String,
    pub title: String,
    pub channel: String,
    pub owner: String,
    pub holder: Option<String>,
    pub lease: Option<Lease>,
    pub changed_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct Lease {
    pub since: Option<String>,
    pub until: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct TaskWindow {
    pub total: u32,
    pub start: u32,
    pub rows: Vec<TaskRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct TaskCount {
    pub state: String,
    pub count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
pub struct TaskChange {
    pub revision: u32,
}

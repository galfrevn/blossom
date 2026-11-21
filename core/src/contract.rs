use serde::Serialize;
use ts_rs::{Config, TS};

pub fn typescript() -> String {
    let configuration = Config::new();
    let declarations = [
        Person::decl(&configuration),
        OpenBoard::decl(&configuration),
        ReplicaState::decl(&configuration),
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
    Copied { stored: u32 },
}

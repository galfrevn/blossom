use std::fmt;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OptionalExtension, params};
use serde::Deserialize;
use serde_json::Value;

use crate::access::BoardAccess;

pub fn replica_file(replicas_folder: &Path, access: &BoardAccess) -> PathBuf {
    let file_stem: String = access
        .url()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    replicas_folder.join(format!("{file_stem}.sqlite"))
}

pub struct MessageView {
    id: String,
    channel: String,
    thread: String,
    created_at: String,
    version: i64,
    json: String,
}

#[derive(Deserialize)]
struct MessageViewKeys {
    id: String,
    channel: String,
    thread: String,
    #[serde(rename = "createdAt")]
    created_at: String,
    version: i64,
}

impl MessageView {
    pub fn parse(json: &str) -> Result<MessageView, serde_json::Error> {
        let keys: MessageViewKeys = serde_json::from_str(json)?;
        Ok(MessageView {
            id: keys.id,
            channel: keys.channel,
            thread: keys.thread,
            created_at: keys.created_at,
            version: keys.version,
            json: json.to_owned(),
        })
    }

    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn created_at(&self) -> &str {
        &self.created_at
    }
}

#[derive(Debug)]
pub enum ReplicaError {
    Storage(rusqlite::Error),
    StoredViewIsNotJson(serde_json::Error),
}

impl fmt::Display for ReplicaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReplicaError::Storage(error) => {
                write!(formatter, "the replica's storage failed: {error}")
            }
            ReplicaError::StoredViewIsNotJson(error) => {
                write!(formatter, "a view in the replica is not json: {error}")
            }
        }
    }
}

impl std::error::Error for ReplicaError {}

impl From<rusqlite::Error> for ReplicaError {
    fn from(error: rusqlite::Error) -> ReplicaError {
        ReplicaError::Storage(error)
    }
}

pub struct Replica {
    connection: Connection,
}

impl Replica {
    pub fn open(file: &Path) -> Result<Replica, ReplicaError> {
        let connection = Connection::open(file)?;
        connection.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             CREATE TABLE IF NOT EXISTS messages (
                 id TEXT PRIMARY KEY,
                 channel TEXT NOT NULL,
                 thread TEXT NOT NULL,
                 created_at TEXT NOT NULL,
                 version INTEGER NOT NULL,
                 view TEXT NOT NULL
             ) WITHOUT ROWID;",
        )?;
        Ok(Replica { connection })
    }

    pub fn apply(&mut self, views: &[MessageView]) -> Result<(), ReplicaError> {
        let transaction = self.connection.transaction()?;
        {
            let mut store_view = transaction.prepare(
                "INSERT INTO messages (id, channel, thread, created_at, version, view)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT (id) DO UPDATE SET
                     channel = excluded.channel,
                     thread = excluded.thread,
                     created_at = excluded.created_at,
                     version = excluded.version,
                     view = excluded.view
                 WHERE excluded.version > messages.version",
            )?;
            for view in views {
                store_view.execute(params![
                    view.id,
                    view.channel,
                    view.thread,
                    view.created_at,
                    view.version,
                    view.json
                ])?;
            }
        }
        transaction.commit()?;
        Ok(())
    }

    pub fn message_count(&self) -> Result<u32, ReplicaError> {
        let count = self
            .connection
            .query_row("SELECT count(*) FROM messages", [], |row| row.get(0))?;
        Ok(count)
    }

    pub fn stored_view(&self, id: &str) -> Result<Option<Value>, ReplicaError> {
        let stored_json: Option<String> = self
            .connection
            .query_row(
                "SELECT view FROM messages WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .optional()?;
        let Some(stored_json) = stored_json else {
            return Ok(None);
        };
        serde_json::from_str(&stored_json)
            .map(Some)
            .map_err(ReplicaError::StoredViewIsNotJson)
    }
}

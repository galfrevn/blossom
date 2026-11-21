use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::{Connection, OptionalExtension, Transaction, params};
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

const WAIT_FOR_ANOTHER_CONNECTION: Duration = Duration::from_secs(2);

pub struct Replica {
    connection: Connection,
    message_count: u32,
}

impl Replica {
    pub fn open(file: &Path) -> Result<Replica, ReplicaError> {
        let connection = Connection::open(file)?;
        connection.busy_timeout(WAIT_FOR_ANOTHER_CONNECTION)?;
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
             ) WITHOUT ROWID;
             CREATE TABLE IF NOT EXISTS feed (
                 only_row INTEGER PRIMARY KEY CHECK (only_row = 1),
                 cursor TEXT NOT NULL
             );",
        )?;
        let message_count =
            connection.query_row("SELECT count(*) FROM messages", [], |row| row.get(0))?;
        Ok(Replica {
            connection,
            message_count,
        })
    }

    pub fn apply(&mut self, views: &[MessageView]) -> Result<(), ReplicaError> {
        let transaction = self.connection.transaction()?;
        let new_messages = store_views(&transaction, views)?;
        transaction.commit()?;
        self.message_count = self.message_count.saturating_add(new_messages);
        Ok(())
    }

    pub fn apply_from_feed(
        &mut self,
        views: &[MessageView],
        feed_cursor: &str,
    ) -> Result<(), ReplicaError> {
        let transaction = self.connection.transaction()?;
        let new_messages = store_views(&transaction, views)?;
        transaction.execute(
            "INSERT INTO feed (only_row, cursor) VALUES (1, ?1)
             ON CONFLICT (only_row) DO UPDATE SET cursor = excluded.cursor",
            params![feed_cursor],
        )?;
        transaction.commit()?;
        self.message_count = self.message_count.saturating_add(new_messages);
        Ok(())
    }

    pub fn feed_cursor(&self) -> Result<Option<String>, ReplicaError> {
        let cursor = self
            .connection
            .query_row("SELECT cursor FROM feed WHERE only_row = 1", [], |row| {
                row.get(0)
            })
            .optional()?;
        Ok(cursor)
    }

    pub fn message_count(&self) -> u32 {
        self.message_count
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

fn store_views(transaction: &Transaction<'_>, views: &[MessageView]) -> Result<u32, ReplicaError> {
    let mut store_new_view = transaction.prepare_cached(
        "INSERT INTO messages (id, channel, thread, created_at, version, view)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT (id) DO NOTHING",
    )?;
    let mut replace_with_newer_view = transaction.prepare_cached(
        "UPDATE messages
         SET channel = ?2, thread = ?3, created_at = ?4, version = ?5, view = ?6
         WHERE id = ?1 AND version < ?5",
    )?;
    let mut new_messages = 0;
    for view in views {
        let fields = params![
            view.id,
            view.channel,
            view.thread,
            view.created_at,
            view.version,
            view.json
        ];
        let stored_as_new = store_new_view.execute(fields)? == 1;
        if stored_as_new {
            new_messages += 1;
        } else {
            replace_with_newer_view.execute(fields)?;
        }
    }
    Ok(new_messages)
}

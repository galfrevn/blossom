mod actors;
mod batch;
mod tasks;

use std::cmp::Reverse;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use serde::Deserialize;
use serde::de::IgnoredAny;
use serde_json::Value;

use crate::access::BoardAccess;
use actors::{ActorDetails, AuthorKeys};
use tasks::RootTask;

pub use actors::{Actor, Via};
pub use batch::Batch;
pub use tasks::{Claim, Hold, Task, TaskEvent};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardIdentity {
    pub oldest_id: String,
    pub oldest_created_at: String,
}

const REPLICA_EXTENSION: &str = ".sqlite";
const NEW_REPLICA_NAME: &str = "new";

pub fn replica_file_for(
    replicas_folder: &Path,
    access: &BoardAccess,
    identity: &BoardIdentity,
) -> PathBuf {
    let address = file_name_part(access.url());
    let created_at = file_name_part(&identity.oldest_created_at);
    let id = file_name_part(&identity.oldest_id);
    replicas_folder.join(format!("{address}.{created_at}.{id}{REPLICA_EXTENSION}"))
}

pub fn new_replica_file(replicas_folder: &Path, access: &BoardAccess) -> PathBuf {
    let address = file_name_part(access.url());
    replicas_folder.join(format!("{address}.{NEW_REPLICA_NAME}{REPLICA_EXTENSION}"))
}

pub fn known_replica_files(replicas_folder: &Path, access: &BoardAccess) -> Vec<PathBuf> {
    let address_prefix = format!("{}.", file_name_part(access.url()));
    let new_replica = new_replica_file(replicas_folder, access);
    let Ok(entries) = fs::read_dir(replicas_folder) else {
        return Vec::new();
    };
    let mut known: Vec<(SystemTime, PathBuf)> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|file| *file != new_replica)
        .filter(|file| {
            file.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    name.starts_with(&address_prefix) && name.ends_with(REPLICA_EXTENSION)
                })
        })
        .map(|file| {
            let modified = fs::metadata(&file)
                .and_then(|metadata| metadata.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            (modified, file)
        })
        .collect();
    known.sort_by_key(|(modified, _)| Reverse(*modified));
    known.into_iter().map(|(_, file)| file).collect()
}

fn file_name_part(text: &str) -> String {
    text.chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

pub struct MessageView {
    id: String,
    channel: String,
    thread: String,
    created_at: String,
    version: i64,
    json: String,
    task: Option<RootTask>,
    author: Option<Actor>,
    re: Option<String>,
    author_actor: String,
    plain_agent_reply: bool,
}

#[derive(Deserialize)]
struct MessageViewKeys {
    id: String,
    channel: String,
    thread: String,
    #[serde(rename = "createdAt")]
    created_at: String,
    version: i64,
    task: Option<RootTask>,
    by: Option<AuthorKeys>,
    author: Option<ActorDetails>,
    re: Option<String>,
    text: Option<String>,
    attachments: Option<Vec<IgnoredAny>>,
}

const MENTION_START: &str = "@[";

impl MessageViewKeys {
    fn author_actor(&self) -> String {
        let actor = self.by.as_ref().map(|by| by.actor.as_str());
        let handle = self.author.as_ref().map(ActorDetails::handle);
        actor.or(handle).unwrap_or_default().to_owned()
    }

    fn is_a_plain_agent_reply(&self) -> bool {
        let is_a_reply = self.re.is_some();
        let written_by_an_agent = self
            .author
            .as_ref()
            .is_some_and(|author| author.kind() == "agent");
        let mentions_someone = self
            .text
            .as_deref()
            .is_some_and(|text| text.contains(MENTION_START));
        let carries_files = self
            .attachments
            .as_ref()
            .is_some_and(|attachments| !attachments.is_empty());
        is_a_reply && written_by_an_agent && !mentions_someone && !carries_files
    }
}

impl MessageView {
    pub fn parse(json: &str) -> Result<MessageView, serde_json::Error> {
        let keys: MessageViewKeys = serde_json::from_str(json)?;
        let author_actor = keys.author_actor();
        let plain_agent_reply = keys.is_a_plain_agent_reply();
        let author = match (keys.by, keys.author) {
            (Some(by), Some(details)) => Some(details.of_actor(by.actor)),
            _ => None,
        };
        Ok(MessageView {
            id: keys.id,
            channel: keys.channel,
            thread: keys.thread,
            created_at: keys.created_at,
            version: keys.version,
            json: json.to_owned(),
            task: keys.task,
            author,
            re: keys.re,
            author_actor,
            plain_agent_reply,
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
    AnotherSchemaVersion(i64),
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
            ReplicaError::AnotherSchemaVersion(found_version) => {
                write!(
                    formatter,
                    "the replica was made with schema version {found_version}, and this core reads only version {SCHEMA_VERSION}"
                )
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

const SCHEMA_VERSION: i64 = 3;

const SCHEMA: &str = "
    CREATE TABLE messages (
        id TEXT PRIMARY KEY,
        channel TEXT NOT NULL,
        thread TEXT NOT NULL,
        created_at TEXT NOT NULL,
        version INTEGER NOT NULL,
        view TEXT NOT NULL,
        re TEXT,
        author TEXT NOT NULL,
        plain_agent_reply INTEGER NOT NULL
    ) WITHOUT ROWID;
    CREATE TABLE feed (
        only_row INTEGER PRIMARY KEY CHECK (only_row = 1),
        cursor TEXT NOT NULL
    );
    CREATE INDEX messages_by_time
        ON messages (created_at, id, channel, author, plain_agent_reply, re);
    CREATE INDEX messages_by_thread ON messages (thread, created_at);
    CREATE TABLE arrivals (
        sequence INTEGER PRIMARY KEY,
        id TEXT NOT NULL
    );
    CREATE TRIGGER arrival_of_a_new_message AFTER INSERT ON messages
    BEGIN
        INSERT INTO arrivals (id) VALUES (new.id);
    END;
    CREATE TABLE channels (
        path TEXT PRIMARY KEY
    ) WITHOUT ROWID;
    CREATE TRIGGER channel_of_a_new_message AFTER INSERT ON messages
    BEGIN
        INSERT OR IGNORE INTO channels (path) VALUES (new.channel);
    END;
    CREATE TABLE board (
        only_row INTEGER PRIMARY KEY CHECK (only_row = 1),
        oldest_id TEXT NOT NULL,
        oldest_created_at TEXT NOT NULL
    );
    CREATE TABLE tasks (
        id TEXT PRIMARY KEY,
        channel TEXT NOT NULL,
        state TEXT NOT NULL,
        owner TEXT NOT NULL,
        made_as TEXT NOT NULL,
        assigned_to TEXT,
        claim_actor TEXT,
        claim_until TEXT,
        needs TEXT NOT NULL,
        blocked_kind TEXT,
        blocked_reason TEXT,
        result TEXT,
        reason TEXT,
        latest_event_at TEXT
    ) WITHOUT ROWID;
    CREATE TABLE task_events (
        position INTEGER PRIMARY KEY,
        cursor TEXT NOT NULL UNIQUE,
        event_type TEXT NOT NULL,
        task TEXT NOT NULL,
        actor TEXT,
        created_at TEXT NOT NULL,
        reason TEXT,
        result TEXT,
        assigned_to TEXT,
        needs TEXT,
        payload TEXT NOT NULL
    );
    CREATE INDEX task_events_by_task ON task_events (task, position);
    CREATE TABLE actors (
        id TEXT PRIMARY KEY,
        kind TEXT NOT NULL,
        handle TEXT NOT NULL,
        name TEXT NOT NULL,
        via_name TEXT,
        via_environment TEXT
    ) WITHOUT ROWID;
    CREATE TABLE unlisted_actors (
        id TEXT PRIMARY KEY
    ) WITHOUT ROWID;
";

fn make_or_check_the_schema(connection: &mut Connection) -> Result<(), ReplicaError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let found_version: i64 =
        transaction.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if found_version == SCHEMA_VERSION {
        return Ok(());
    }
    let holds_any_table: bool =
        transaction.query_row("SELECT EXISTS (SELECT 1 FROM sqlite_schema)", [], |row| {
            row.get(0)
        })?;
    if holds_any_table {
        return Err(ReplicaError::AnotherSchemaVersion(found_version));
    }
    transaction.execute_batch(SCHEMA)?;
    transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    transaction.commit()?;
    Ok(())
}

pub struct Replica {
    connection: Connection,
    message_count: u32,
}

impl Replica {
    pub fn open(file: &Path) -> Result<Replica, ReplicaError> {
        let mut connection = Connection::open(file)?;
        connection.busy_timeout(WAIT_FOR_ANOTHER_CONNECTION)?;
        connection.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;",
        )?;
        make_or_check_the_schema(&mut connection)?;
        let message_count =
            connection.query_row("SELECT count(*) FROM messages", [], |row| row.get(0))?;
        Ok(Replica {
            connection,
            message_count,
        })
    }

    pub fn apply(&mut self, batch: &Batch) -> Result<(), ReplicaError> {
        let transaction = self.connection.transaction()?;
        let new_messages = store_batch(&transaction, batch)?;
        transaction.commit()?;
        self.message_count = self.message_count.saturating_add(new_messages);
        Ok(())
    }

    pub fn apply_from_feed(
        &mut self,
        batch: &Batch,
        feed_cursor: &str,
    ) -> Result<(), ReplicaError> {
        let transaction = self.connection.transaction()?;
        let new_messages = store_batch(&transaction, batch)?;
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

    pub fn identity(&self) -> Result<Option<BoardIdentity>, ReplicaError> {
        let identity = self
            .connection
            .query_row(
                "SELECT oldest_id, oldest_created_at FROM board WHERE only_row = 1",
                [],
                |row| {
                    Ok(BoardIdentity {
                        oldest_id: row.get(0)?,
                        oldest_created_at: row.get(1)?,
                    })
                },
            )
            .optional()?;
        Ok(identity)
    }

    pub fn keep_identity(&mut self, identity: &BoardIdentity) -> Result<(), ReplicaError> {
        self.connection.execute(
            "INSERT INTO board (only_row, oldest_id, oldest_created_at) VALUES (1, ?1, ?2)
             ON CONFLICT (only_row) DO UPDATE SET
                 oldest_id = excluded.oldest_id,
                 oldest_created_at = excluded.oldest_created_at",
            params![identity.oldest_id, identity.oldest_created_at],
        )?;
        Ok(())
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

    pub fn task(&self, id: &str) -> Result<Option<Task>, ReplicaError> {
        Ok(tasks::task(&self.connection, id)?)
    }

    pub fn task_events(&self, task: &str) -> Result<Vec<TaskEvent>, ReplicaError> {
        Ok(tasks::task_events(&self.connection, task)?)
    }

    pub fn actor(&self, id: &str) -> Result<Option<Actor>, ReplicaError> {
        Ok(actors::actor(&self.connection, id)?)
    }

    pub(crate) fn unseen_actors(&self, named_ids: &[String]) -> Result<Vec<String>, ReplicaError> {
        Ok(actors::unseen_actors(&self.connection, named_ids)?)
    }
}

fn store_batch(transaction: &Transaction<'_>, batch: &Batch) -> Result<u32, ReplicaError> {
    let new_messages = store_views(transaction, &batch.views)?;
    tasks::keep_task_events(transaction, &batch.task_events)?;
    actors::keep_actors(transaction, batch.actors.values())?;
    actors::keep_unlisted_actors(transaction, &batch.unlisted_actor_ids)?;
    Ok(new_messages)
}

fn store_views(transaction: &Transaction<'_>, views: &[MessageView]) -> Result<u32, ReplicaError> {
    let mut store_new_view = transaction.prepare_cached(
        "INSERT INTO messages (
             id, channel, thread, created_at, version, view, re, author, plain_agent_reply
         )
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT (id) DO NOTHING",
    )?;
    let mut replace_with_newer_view = transaction.prepare_cached(
        "UPDATE messages
         SET channel = ?2, thread = ?3, created_at = ?4, version = ?5, view = ?6,
             re = ?7, author = ?8, plain_agent_reply = ?9
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
            view.json,
            view.re,
            view.author_actor,
            view.plain_agent_reply
        ];
        let stored_as_new = store_new_view.execute(fields)? == 1;
        if stored_as_new {
            new_messages += 1;
        }
        let view_is_the_newest = stored_as_new || replace_with_newer_view.execute(fields)? == 1;
        let Some(task) = &view.task else {
            continue;
        };
        if view_is_the_newest {
            tasks::keep_root_task(transaction, &view.id, &view.channel, task)?;
        }
    }
    Ok(new_messages)
}

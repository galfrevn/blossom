use rusqlite::types::Type;
use rusqlite::{Connection, OptionalExtension, Row, Transaction, params};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use super::actors::{Actor, AuthorKeys};

pub(crate) const TASK_EVENT_PREFIX: &str = "task.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub id: String,
    pub channel: String,
    pub state: String,
    pub owner: String,
    pub made_as: String,
    pub assigned_to: Option<String>,
    pub claim: Option<Claim>,
    pub needs: Vec<String>,
    pub blocked: Option<Hold>,
    pub result: Option<String>,
    pub reason: Option<String>,
    pub latest_event_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Claim {
    pub actor: String,
    pub until: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Hold {
    pub kind: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TaskEvent {
    pub cursor: String,
    pub event_type: String,
    pub task: String,
    pub actor: Option<String>,
    pub created_at: String,
    pub reason: Option<String>,
    pub result: Option<String>,
    pub assigned_to: Option<String>,
    pub needs: Option<Vec<String>>,
    pub payload: Value,
}

#[derive(Deserialize)]
pub(crate) struct RootTask {
    state: String,
    owner: String,
    #[serde(rename = "by")]
    made_as: String,
    #[serde(rename = "for")]
    assigned_to: Option<String>,
    needs: Vec<String>,
    claim: Option<Claim>,
    blocked: Option<Hold>,
    result: Option<String>,
    reason: Option<String>,
}

impl RootTask {
    pub(crate) fn actor_ids(&self) -> impl Iterator<Item = &str> {
        let claim_actor = self.claim.as_ref().map(|claim| claim.actor.as_str());
        [
            Some(self.owner.as_str()),
            Some(self.made_as.as_str()),
            self.assigned_to.as_deref(),
            claim_actor,
        ]
        .into_iter()
        .flatten()
    }
}

#[derive(Deserialize)]
struct TaskEventKeys {
    cursor: String,
    #[serde(rename = "createdAt")]
    created_at: String,
    payload: Value,
}

#[derive(Deserialize)]
struct TaskPayloadKeys {
    id: String,
    by: Option<AuthorKeys>,
    #[serde(rename = "for")]
    assigned_to: Option<String>,
    needs: Option<Vec<String>>,
    reason: Option<String>,
    result: Option<String>,
}

impl TaskEvent {
    pub(crate) fn parse(
        event_json: &str,
        event_type: &str,
        actor_the_event_names: Option<&Actor>,
    ) -> Result<TaskEvent, serde_json::Error> {
        let keys: TaskEventKeys = serde_json::from_str(event_json)?;
        let payload = TaskPayloadKeys::deserialize(&keys.payload)?;
        let actor_the_payload_names = payload.by.map(|by| by.actor);
        let actor = actor_the_event_names
            .map(|actor| actor.id.clone())
            .or(actor_the_payload_names);
        Ok(TaskEvent {
            cursor: keys.cursor,
            event_type: event_type.to_owned(),
            task: payload.id,
            actor,
            created_at: keys.created_at,
            reason: payload.reason,
            result: payload.result,
            assigned_to: payload.assigned_to,
            needs: payload.needs,
            payload: keys.payload,
        })
    }

    pub(crate) fn actor_ids(&self) -> impl Iterator<Item = &str> {
        [self.actor.as_deref(), self.assigned_to.as_deref()]
            .into_iter()
            .flatten()
    }
}

fn needs_as_json(needs: &[String]) -> String {
    Value::from(needs).to_string()
}

fn parse_json_column<Parsed: DeserializeOwned>(
    column: usize,
    json: &str,
) -> rusqlite::Result<Parsed> {
    serde_json::from_str(json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(column, Type::Text, Box::new(error))
    })
}

pub(super) fn keep_root_task(
    transaction: &Transaction<'_>,
    id: &str,
    channel: &str,
    task: &RootTask,
) -> rusqlite::Result<()> {
    let mut keep_newest_task = transaction.prepare_cached(
        "INSERT INTO tasks (
             id, channel, state, owner, made_as, assigned_to, claim_actor, claim_until, needs,
             blocked_kind, blocked_reason, result, reason, latest_event_at
         )
         VALUES (
             ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13,
             (SELECT max(created_at) FROM task_events WHERE task = ?1)
         )
         ON CONFLICT (id) DO UPDATE SET
             channel = excluded.channel,
             state = excluded.state,
             owner = excluded.owner,
             made_as = excluded.made_as,
             assigned_to = excluded.assigned_to,
             claim_actor = excluded.claim_actor,
             claim_until = excluded.claim_until,
             needs = excluded.needs,
             blocked_kind = excluded.blocked_kind,
             blocked_reason = excluded.blocked_reason,
             result = excluded.result,
             reason = excluded.reason",
    )?;
    let claim_actor = task.claim.as_ref().map(|claim| claim.actor.as_str());
    let claim_until = task.claim.as_ref().map(|claim| claim.until.as_str());
    let blocked_kind = task.blocked.as_ref().and_then(|hold| hold.kind.as_deref());
    let blocked_reason = task.blocked.as_ref().map(|hold| hold.reason.as_str());
    keep_newest_task.execute(params![
        id,
        channel,
        task.state,
        task.owner,
        task.made_as,
        task.assigned_to,
        claim_actor,
        claim_until,
        needs_as_json(&task.needs),
        blocked_kind,
        blocked_reason,
        task.result,
        task.reason
    ])?;
    Ok(())
}

pub(super) fn keep_task_events(
    transaction: &Transaction<'_>,
    events: &[TaskEvent],
) -> rusqlite::Result<()> {
    let mut keep_event_once = transaction.prepare_cached(
        "INSERT INTO task_events (
             cursor, event_type, task, actor, created_at, reason, result, assigned_to, needs,
             payload
         )
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT (cursor) DO NOTHING",
    )?;
    let mut move_the_latest_event_time = transaction.prepare_cached(
        "UPDATE tasks SET latest_event_at = ?2
         WHERE id = ?1 AND (latest_event_at IS NULL OR latest_event_at < ?2)",
    )?;
    for event in events {
        let needs = event.needs.as_deref().map(needs_as_json);
        let kept_for_the_first_time = keep_event_once.execute(params![
            event.cursor,
            event.event_type,
            event.task,
            event.actor,
            event.created_at,
            event.reason,
            event.result,
            event.assigned_to,
            needs,
            event.payload.to_string()
        ])? == 1;
        if kept_for_the_first_time {
            move_the_latest_event_time.execute(params![event.task, event.created_at])?;
        }
    }
    Ok(())
}

fn task_from_row(row: &Row<'_>) -> rusqlite::Result<Task> {
    let claim_actor: Option<String> = row.get(6)?;
    let claim_until: Option<String> = row.get(7)?;
    let claim = claim_actor
        .zip(claim_until)
        .map(|(actor, until)| Claim { actor, until });
    let needs_json: String = row.get(8)?;
    let blocked_kind: Option<String> = row.get(9)?;
    let blocked_reason: Option<String> = row.get(10)?;
    let blocked = blocked_reason.map(|reason| Hold {
        kind: blocked_kind,
        reason,
    });
    Ok(Task {
        id: row.get(0)?,
        channel: row.get(1)?,
        state: row.get(2)?,
        owner: row.get(3)?,
        made_as: row.get(4)?,
        assigned_to: row.get(5)?,
        claim,
        needs: parse_json_column(8, &needs_json)?,
        blocked,
        result: row.get(11)?,
        reason: row.get(12)?,
        latest_event_at: row.get(13)?,
    })
}

pub(super) fn task(connection: &Connection, id: &str) -> rusqlite::Result<Option<Task>> {
    connection
        .prepare_cached(
            "SELECT id, channel, state, owner, made_as, assigned_to, claim_actor, claim_until,
                    needs, blocked_kind, blocked_reason, result, reason, latest_event_at
             FROM tasks WHERE id = ?1",
        )?
        .query_row(params![id], task_from_row)
        .optional()
}

fn task_event_from_row(row: &Row<'_>) -> rusqlite::Result<TaskEvent> {
    let needs_json: Option<String> = row.get(8)?;
    let needs = needs_json
        .map(|needs_json| parse_json_column(8, &needs_json))
        .transpose()?;
    let payload_json: String = row.get(9)?;
    Ok(TaskEvent {
        cursor: row.get(0)?,
        event_type: row.get(1)?,
        task: row.get(2)?,
        actor: row.get(3)?,
        created_at: row.get(4)?,
        reason: row.get(5)?,
        result: row.get(6)?,
        assigned_to: row.get(7)?,
        needs,
        payload: parse_json_column(9, &payload_json)?,
    })
}

pub(super) fn task_events(connection: &Connection, task: &str) -> rusqlite::Result<Vec<TaskEvent>> {
    connection
        .prepare_cached(
            "SELECT cursor, event_type, task, actor, created_at, reason, result, assigned_to,
                    needs, payload
             FROM task_events WHERE task = ?1 ORDER BY position",
        )?
        .query_map(params![task], task_event_from_row)?
        .collect()
}

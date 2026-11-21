use rusqlite::{Connection, Row, params};

use crate::contract::{Agent, HeldTask};

const EVERY_AGENT_WITH_ITS_LATEST_DOINGS: &str = "
    WITH
        moment (an_hour_ago) AS (
            SELECT strftime('%Y-%m-%dT%H:%M:%fZ', ?1, '-1 hour')
        ),
        agents AS MATERIALIZED (
            SELECT
                actors.id,
                actors.handle,
                actors.name,
                actors.via_name,
                (SELECT tasks.id FROM tasks
                 WHERE tasks.claim_actor = actors.id AND tasks.state = 'claimed'
                 ORDER BY tasks.claim_until DESC
                 LIMIT 1) AS held_task,
                (SELECT claims.position FROM task_events AS claims
                 WHERE claims.event_type = 'task.claimed' AND claims.actor = actors.id
                 ORDER BY claims.created_at DESC, claims.position DESC
                 LIMIT 1) AS latest_claim,
                (SELECT messages.id FROM messages
                 WHERE messages.author = actors.id
                 ORDER BY messages.created_at DESC
                 LIMIT 1) AS latest_message
            FROM actors
            WHERE actors.kind = 'agent'
        )
    SELECT
        agents.id AS id,
        agents.name AS name,
        agents.via_name AS on_behalf_of,
        held.id AS held_task,
        held.claim_until AS held_until,
        EXISTS (
            SELECT 1 FROM task_events AS expiry
            WHERE expiry.task = latest_claim.task
              AND expiry.position > latest_claim.position
              AND expiry.event_type = 'task.expired'
              AND json_extract(expiry.payload, '$.epoch')
                  = json_extract(latest_claim.payload, '$.epoch')
        ) AS latest_claim_expired,
        EXISTS (
            SELECT 1 FROM task_events AS failure
            WHERE failure.event_type = 'task.failed'
              AND failure.actor = agents.id
              AND failure.created_at >= moment.an_hour_ago
        ) AS failed_recently,
        latest_claim.created_at AS latest_claim_at,
        claimed.channel AS latest_claim_channel,
        latest_message.created_at AS latest_message_at,
        latest_message.channel AS latest_message_channel
    FROM agents
    CROSS JOIN moment
    LEFT JOIN tasks AS held ON held.id = agents.held_task
    LEFT JOIN task_events AS latest_claim ON latest_claim.position = agents.latest_claim
    LEFT JOIN tasks AS claimed ON claimed.id = latest_claim.task
    LEFT JOIN messages AS latest_message ON latest_message.id = agents.latest_message
    ORDER BY agents.handle, agents.id
";

struct Sighting {
    at: String,
    channel: String,
}

impl Sighting {
    fn from_columns(at: Option<String>, channel: Option<String>) -> Option<Sighting> {
        at.zip(channel)
            .map(|(at, channel)| Sighting { at, channel })
    }
}

fn channel_of_the_later(
    latest_claim: Option<Sighting>,
    latest_message: Option<Sighting>,
) -> Option<String> {
    match (latest_claim, latest_message) {
        (Some(claim), Some(message)) if message.at > claim.at => Some(message.channel),
        (Some(claim), _) => Some(claim.channel),
        (None, message) => message.map(|message| message.channel),
    }
}

fn agent_from_row(row: &Row<'_>, now: &str) -> rusqlite::Result<Agent> {
    let held_task: Option<String> = row.get("held_task")?;
    let held_until: Option<String> = row.get("held_until")?;
    let holds = held_task
        .zip(held_until)
        .map(|(task, until)| HeldTask { task, until });
    let lease_passed = holds.as_ref().is_some_and(|held| held.until.as_str() < now);
    let latest_claim_expired: bool = row.get("latest_claim_expired")?;
    let latest_claim = Sighting::from_columns(
        row.get("latest_claim_at")?,
        row.get("latest_claim_channel")?,
    );
    let latest_message = Sighting::from_columns(
        row.get("latest_message_at")?,
        row.get("latest_message_channel")?,
    );
    Ok(Agent {
        id: row.get("id")?,
        name: row.get("name")?,
        on_behalf_of: row.get("on_behalf_of")?,
        holds,
        lapsed: latest_claim_expired || lease_passed,
        failed_recently: row.get("failed_recently")?,
        channel: channel_of_the_later(latest_claim, latest_message),
    })
}

pub(super) fn fleet(connection: &Connection, now: &str) -> rusqlite::Result<Vec<Agent>> {
    let mut statement = connection.prepare_cached(EVERY_AGENT_WITH_ITS_LATEST_DOINGS)?;
    let agents = statement.query_map(params![now], |row| agent_from_row(row, now))?;
    agents.collect()
}

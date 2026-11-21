use std::path::Path;

use rusqlite::{Connection, OpenFlags, params};

use crate::contract::{FlowRow, FlowWindow};
use crate::replica::ReplicaError;

const MINUTE_PREFIX_LENGTH: usize = "2026-11-21T10:00".len();

struct OrderedMessage {
    created_at: Box<str>,
    id: Box<str>,
}

impl OrderedMessage {
    fn minute(&self) -> &str {
        self.created_at
            .get(..MINUTE_PREFIX_LENGTH)
            .unwrap_or(&self.created_at)
    }

    fn comes_before(&self, created_at: &str, id: &str) -> bool {
        (&*self.created_at, &*self.id) < (created_at, id)
    }
}

pub struct ReplicaReader {
    connection: Connection,
    flow: Vec<OrderedMessage>,
    last_arrival: i64,
}

impl ReplicaReader {
    pub fn open(file: &Path) -> Result<ReplicaReader, ReplicaError> {
        let connection = Connection::open_with_flags(
            file,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        let snapshot = connection.unchecked_transaction()?;
        let last_arrival: i64 = snapshot.query_row(
            "SELECT coalesce(max(sequence), 0) FROM arrivals",
            [],
            |row| row.get(0),
        )?;
        let mut flow = Vec::new();
        {
            let mut statement =
                snapshot.prepare("SELECT created_at, id FROM messages ORDER BY created_at, id")?;
            let rows = statement.query_map([], |row| {
                Ok(OrderedMessage {
                    created_at: row.get::<_, String>(0)?.into_boxed_str(),
                    id: row.get::<_, String>(1)?.into_boxed_str(),
                })
            })?;
            for row in rows {
                flow.push(row?);
            }
        }
        snapshot.commit()?;
        Ok(ReplicaReader {
            connection,
            flow,
            last_arrival,
        })
    }

    pub fn is_read_only(&self) -> bool {
        self.connection
            .is_readonly(rusqlite::MAIN_DB)
            .unwrap_or(false)
    }

    pub fn channels(&self) -> Result<Vec<String>, ReplicaError> {
        let mut statement = self
            .connection
            .prepare_cached("SELECT path FROM channels ORDER BY path")?;
        let paths = statement.query_map([], |row| row.get(0))?;
        let mut channels = Vec::new();
        for path in paths {
            channels.push(path?);
        }
        Ok(channels)
    }

    pub fn flow_total(&self) -> u32 {
        u32::try_from(self.flow.len()).unwrap_or(u32::MAX)
    }

    pub fn catch_up(&mut self) -> Result<bool, ReplicaError> {
        let mut statement = self.connection.prepare_cached(
            "SELECT arrivals.sequence, messages.created_at, messages.id
             FROM arrivals JOIN messages ON messages.id = arrivals.id
             WHERE arrivals.sequence > ?1
             ORDER BY arrivals.sequence",
        )?;
        let arrived = statement.query_map(params![self.last_arrival], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;
        let mut caught_up = false;
        for arrival in arrived {
            let (sequence, created_at, id) = arrival?;
            let place = self
                .flow
                .partition_point(|ordered| ordered.comes_before(&created_at, &id));
            self.flow.insert(
                place,
                OrderedMessage {
                    created_at: created_at.into_boxed_str(),
                    id: id.into_boxed_str(),
                },
            );
            self.last_arrival = sequence;
            caught_up = true;
        }
        Ok(caught_up)
    }

    pub fn flow_window(&self, start: u32, count: u32) -> Result<FlowWindow, ReplicaError> {
        let total = self.flow.len();
        let first = usize::try_from(start).unwrap_or(usize::MAX).min(total);
        let wanted = usize::try_from(count).unwrap_or(usize::MAX);
        let last = first.saturating_add(wanted).min(total);
        let mut statement = self.connection.prepare_cached(
            "SELECT channel,
                    coalesce(json_extract(view, '$.author.name'), json_extract(view, '$.by.actor'), ''),
                    json_extract(view, '$.author.via.name'),
                    coalesce(json_extract(view, '$.text'), ''),
                    json_extract(view, '$.editedAt') IS NOT NULL,
                    coalesce(json_extract(view, '$.deleted'), 0)
             FROM messages WHERE id = ?1",
        )?;
        let mut rows = Vec::with_capacity(last - first);
        for position in first..last {
            let Some(ordered) = self.flow.get(position) else {
                break;
            };
            let previous_minute = position
                .checked_sub(1)
                .and_then(|previous| self.flow.get(previous))
                .map(OrderedMessage::minute);
            let first_of_its_minute = previous_minute != Some(ordered.minute());
            let row = statement.query_row(params![&*ordered.id], |row| {
                Ok(FlowRow {
                    id: ordered.id.to_string(),
                    channel: row.get(0)?,
                    author: row.get(1)?,
                    on_behalf_of: row.get(2)?,
                    created_at: ordered.created_at.to_string(),
                    text: row.get(3)?,
                    edited: row.get(4)?,
                    deleted: row.get(5)?,
                    first_of_its_minute,
                })
            })?;
            rows.push(row);
        }
        Ok(FlowWindow {
            total: u32::try_from(total).unwrap_or(u32::MAX),
            start: u32::try_from(first).unwrap_or(u32::MAX),
            rows,
        })
    }
}

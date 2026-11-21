use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;

use rusqlite::{CachedStatement, Connection, OpenFlags, params};

use crate::contract::{FlowRow, FlowWindow};
use crate::replica::ReplicaError;

const MINUTE_PREFIX_LENGTH: usize = "2026-11-21T10:00".len();

struct OrderedMessage {
    created_at: Box<str>,
    id: Box<str>,
    channel: u32,
}

struct Narrowed {
    channel: String,
    flow_length: usize,
    positions: Vec<usize>,
}

#[derive(Default)]
struct ChannelNumbers {
    names: Vec<Box<str>>,
    numbers: HashMap<Box<str>, u32>,
}

impl ChannelNumbers {
    fn number_of(&mut self, channel: &str) -> u32 {
        if let Some(number) = self.numbers.get(channel) {
            return *number;
        }
        let number = u32::try_from(self.names.len()).unwrap_or(u32::MAX);
        self.names.push(channel.into());
        self.numbers.insert(channel.into(), number);
        number
    }

    fn within(&self, channel: &str) -> Vec<bool> {
        let subtree = format!("{channel}/");
        self.names
            .iter()
            .map(|name| &**name == channel || name.starts_with(&subtree))
            .collect()
    }
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
    channel_numbers: ChannelNumbers,
    narrowed: RefCell<Option<Narrowed>>,
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
        let mut channel_numbers = ChannelNumbers::default();
        {
            let mut statement = snapshot
                .prepare("SELECT created_at, id, channel FROM messages ORDER BY created_at, id")?;
            let rows = statement.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?;
            for row in rows {
                let (created_at, id, channel) = row?;
                flow.push(OrderedMessage {
                    created_at: created_at.into_boxed_str(),
                    id: id.into_boxed_str(),
                    channel: channel_numbers.number_of(&channel),
                });
            }
        }
        snapshot.commit()?;
        Ok(ReplicaReader {
            connection,
            flow,
            channel_numbers,
            narrowed: RefCell::new(None),
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

    pub fn catch_up(&mut self) -> Result<bool, ReplicaError> {
        let mut statement = self.connection.prepare_cached(
            "SELECT arrivals.sequence, messages.created_at, messages.id, messages.channel
             FROM arrivals JOIN messages ON messages.id = arrivals.id
             WHERE arrivals.sequence > ?1
             ORDER BY arrivals.sequence",
        )?;
        let arrived = statement.query_map(params![self.last_arrival], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;
        let mut caught_up = false;
        for arrival in arrived {
            let (sequence, created_at, id, channel) = arrival?;
            let channel = self.channel_numbers.number_of(&channel);
            let place = self
                .flow
                .partition_point(|ordered| ordered.comes_before(&created_at, &id));
            self.flow.insert(
                place,
                OrderedMessage {
                    created_at: created_at.into_boxed_str(),
                    id: id.into_boxed_str(),
                    channel,
                },
            );
            self.last_arrival = sequence;
            caught_up = true;
        }
        Ok(caught_up)
    }

    pub fn flow_window_in(
        &self,
        channel: Option<&str>,
        start: u32,
        count: u32,
    ) -> Result<FlowWindow, ReplicaError> {
        let Some(channel) = channel else {
            return self.window_of(self.flow.len(), Some, start, count);
        };
        let mut narrowed = self.narrowed.borrow_mut();
        let is_current = narrowed.as_ref().is_some_and(|narrowed| {
            narrowed.channel == channel && narrowed.flow_length == self.flow.len()
        });
        if !is_current {
            let within = self.channel_numbers.within(channel);
            let positions = self
                .flow
                .iter()
                .enumerate()
                .filter(|(_, ordered)| {
                    within
                        .get(usize::try_from(ordered.channel).unwrap_or(usize::MAX))
                        .copied()
                        .unwrap_or(false)
                })
                .map(|(position, _)| position)
                .collect();
            *narrowed = Some(Narrowed {
                channel: channel.to_owned(),
                flow_length: self.flow.len(),
                positions,
            });
        }
        let positions = narrowed
            .as_ref()
            .map(|narrowed| narrowed.positions.as_slice())
            .unwrap_or_default();
        self.window_of(
            positions.len(),
            |index| positions.get(index).copied(),
            start,
            count,
        )
    }

    fn window_of(
        &self,
        total: usize,
        position_of: impl Fn(usize) -> Option<usize>,
        start: u32,
        count: u32,
    ) -> Result<FlowWindow, ReplicaError> {
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
        for index in first..last {
            let Some(ordered) = position_of(index).and_then(|position| self.flow.get(position))
            else {
                break;
            };
            let previous_minute = index
                .checked_sub(1)
                .and_then(&position_of)
                .and_then(|position| self.flow.get(position))
                .map(OrderedMessage::minute);
            let first_of_its_minute = previous_minute != Some(ordered.minute());
            rows.push(read_row(&mut statement, ordered, first_of_its_minute)?);
        }
        Ok(FlowWindow {
            total: u32::try_from(total).unwrap_or(u32::MAX),
            start: u32::try_from(first).unwrap_or(u32::MAX),
            rows,
        })
    }
}

fn read_row(
    statement: &mut CachedStatement<'_>,
    ordered: &OrderedMessage,
    first_of_its_minute: bool,
) -> Result<FlowRow, ReplicaError> {
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
    Ok(row)
}

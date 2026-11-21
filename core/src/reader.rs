mod fleet;
mod lines;
mod numbers;
mod thread;

use std::cell::RefCell;
use std::collections::HashSet;
use std::path::Path;

use rusqlite::types::Type;
use rusqlite::{CachedStatement, Connection, OpenFlags, Row, params};

use crate::contract::{Agent, FlowFold, FlowRow, FlowWindow};
use crate::replica::ReplicaError;
use lines::{Line, OrderedMessage, lines_of};
use numbers::Numbers;

const MESSAGES_A_FOLD_SHOWS: usize = 8;

const ORDERED_MESSAGE_COLUMNS: &str = "messages.created_at,
     messages.id,
     messages.channel,
     messages.author,
     messages.plain_agent_reply
         AND EXISTS (SELECT 1 FROM tasks WHERE tasks.id = messages.re)";

struct Narrowed {
    channel: String,
    revision: u64,
    lines: Vec<usize>,
}

fn text_at<'row>(row: &'row Row<'_>, column: usize) -> rusqlite::Result<&'row str> {
    row.get_ref(column)?.as_str().map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(column, Type::Text, Box::new(error))
    })
}

fn ordered_message(
    row: &Row<'_>,
    first_column: usize,
    channels: &mut Numbers,
    authors: &mut Numbers,
) -> rusqlite::Result<OrderedMessage> {
    Ok(OrderedMessage {
        created_at: text_at(row, first_column)?.into(),
        id: text_at(row, first_column + 1)?.into(),
        channel: channels.number_of(text_at(row, first_column + 2)?),
        author: authors.number_of(text_at(row, first_column + 3)?),
        progress: row.get(first_column + 4)?,
    })
}

pub struct ReplicaReader {
    connection: Connection,
    flow: Vec<OrderedMessage>,
    lines: Vec<Line>,
    revision: u64,
    channels: Numbers,
    authors: Numbers,
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
        let mut channels = Numbers::default();
        let mut authors = Numbers::default();
        let mut flow = Vec::new();
        {
            let mut statement = snapshot.prepare(&format!(
                "SELECT {ORDERED_MESSAGE_COLUMNS} FROM messages ORDER BY created_at, id"
            ))?;
            let mut rows = statement.query([])?;
            while let Some(row) = rows.next()? {
                flow.push(ordered_message(row, 0, &mut channels, &mut authors)?);
            }
        }
        snapshot.commit()?;
        let lines = lines_of(&flow);
        Ok(ReplicaReader {
            connection,
            flow,
            lines,
            revision: 0,
            channels,
            authors,
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

    pub fn fleet(&self, now: &str) -> Result<Vec<Agent>, ReplicaError> {
        Ok(fleet::fleet(&self.connection, now)?)
    }

    pub fn catch_up(&mut self) -> Result<bool, ReplicaError> {
        let mut arrived = Vec::new();
        {
            let mut statement = self.connection.prepare_cached(&format!(
                "SELECT arrivals.sequence, {ORDERED_MESSAGE_COLUMNS}
                 FROM arrivals JOIN messages ON messages.id = arrivals.id
                 WHERE arrivals.sequence > ?1
                 ORDER BY arrivals.sequence"
            ))?;
            let mut rows = statement.query(params![self.last_arrival])?;
            while let Some(row) = rows.next()? {
                let sequence: i64 = row.get(0)?;
                let ordered = ordered_message(row, 1, &mut self.channels, &mut self.authors)?;
                arrived.push((sequence, ordered));
            }
        }
        if arrived.is_empty() {
            return Ok(false);
        }
        for (sequence, ordered) in arrived {
            let place = self.flow.partition_point(|existing| {
                existing.comes_before(&ordered.created_at, &ordered.id)
            });
            self.flow.insert(place, ordered);
            self.last_arrival = sequence;
        }
        self.lines = lines_of(&self.flow);
        self.revision = self.revision.wrapping_add(1);
        Ok(true)
    }

    pub fn flow_window_in(
        &self,
        channel: Option<&str>,
        start: u32,
        count: u32,
    ) -> Result<FlowWindow, ReplicaError> {
        let Some(channel) = channel else {
            return self.window_of(
                self.lines.len(),
                |index| self.lines.get(index),
                start,
                count,
            );
        };
        let mut narrowed = self.narrowed.borrow_mut();
        let is_current = narrowed.as_ref().is_some_and(|narrowed| {
            narrowed.channel == channel && narrowed.revision == self.revision
        });
        if !is_current {
            let within = self.channels.within(channel);
            let lines = self
                .lines
                .iter()
                .enumerate()
                .filter(|(_, line)| {
                    line.channel(&self.flow)
                        .and_then(|number| within.get(usize::try_from(number).ok()?).copied())
                        .unwrap_or(false)
                })
                .map(|(index, _)| index)
                .collect();
            *narrowed = Some(Narrowed {
                channel: channel.to_owned(),
                revision: self.revision,
                lines,
            });
        }
        let lines = narrowed
            .as_ref()
            .map(|narrowed| narrowed.lines.as_slice())
            .unwrap_or_default();
        self.window_of(
            lines.len(),
            |index| lines.get(index).and_then(|line| self.lines.get(*line)),
            start,
            count,
        )
    }

    fn window_of<'lines>(
        &'lines self,
        total: usize,
        line_at: impl Fn(usize) -> Option<&'lines Line>,
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
            let Some(line) = line_at(index) else {
                break;
            };
            let previous_minute = index
                .checked_sub(1)
                .and_then(&line_at)
                .and_then(|previous| self.message_at(previous.latest()))
                .map(OrderedMessage::minute);
            let Some(latest) = self.message_at(line.latest()) else {
                break;
            };
            let first_of_its_minute = previous_minute != Some(latest.minute());
            let mut row = read_row(&mut statement, latest, first_of_its_minute)?;
            if let Line::Fold { members, .. } = line {
                row.fold = Some(self.fold_of(&mut statement, members)?);
            }
            rows.push(row);
        }
        Ok(FlowWindow {
            total: u32::try_from(total).unwrap_or(u32::MAX),
            start: u32::try_from(first).unwrap_or(u32::MAX),
            rows,
        })
    }

    fn message_at(&self, position: Option<usize>) -> Option<&OrderedMessage> {
        position.and_then(|position| self.flow.get(position))
    }

    fn fold_of(
        &self,
        statement: &mut CachedStatement<'_>,
        members: &[usize],
    ) -> Result<FlowFold, ReplicaError> {
        let agents: HashSet<u32> = members
            .iter()
            .filter_map(|position| self.flow.get(*position))
            .map(|message| message.author)
            .collect();
        let shown_from = members.len().saturating_sub(MESSAGES_A_FOLD_SHOWS);
        let mut recent = Vec::new();
        for position in members.get(shown_from..).unwrap_or_default() {
            if let Some(member) = self.flow.get(*position) {
                recent.push(read_row(statement, member, false)?);
            }
        }
        Ok(FlowFold {
            count: u32::try_from(members.len()).unwrap_or(u32::MAX),
            agents: u32::try_from(agents.len()).unwrap_or(u32::MAX),
            recent,
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
            fold: None,
        })
    })?;
    Ok(row)
}

mod fleet;
mod lines;
mod numbers;
mod search;
mod tasks;
mod thread;

use std::cell::{Ref, RefCell};
use std::collections::HashSet;
use std::path::Path;

use rusqlite::types::Type;
use rusqlite::{CachedStatement, Connection, OpenFlags, Row, params};

use crate::contract::{Agent, FlowFold, FlowRow, FlowWindow, SearchResults};
use crate::replica::ReplicaError;
use lines::{Line, OrderedMessage, lines_of, place_in_order, take_in_changes};
use numbers::Numbers;
use tasks::TaskOrder;

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

fn last_in(connection: &Connection, log: &str) -> rusqlite::Result<i64> {
    connection.query_row(
        &format!("SELECT coalesce(max(sequence), 0) FROM {log}"),
        [],
        |row| row.get(0),
    )
}

struct LoggedSince {
    messages: Vec<OrderedMessage>,
    last: i64,
}

fn messages_logged_since(
    connection: &Connection,
    log: &str,
    last: i64,
    channels: &mut Numbers,
    authors: &mut Numbers,
) -> rusqlite::Result<LoggedSince> {
    let mut statement = connection.prepare_cached(&format!(
        "SELECT {log}.sequence, {ORDERED_MESSAGE_COLUMNS}
         FROM {log} JOIN messages ON messages.id = {log}.id
         WHERE {log}.sequence > ?1
         ORDER BY {log}.sequence"
    ))?;
    let mut logged = LoggedSince {
        messages: Vec::new(),
        last,
    };
    let mut rows = statement.query(params![last])?;
    while let Some(row) = rows.next()? {
        logged.last = row.get(0)?;
        logged
            .messages
            .push(ordered_message(row, 1, channels, authors)?);
    }
    Ok(logged)
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
    last_message_change: i64,
    tasks: TaskOrder,
}

impl ReplicaReader {
    pub fn open(file: &Path) -> Result<ReplicaReader, ReplicaError> {
        let connection = Connection::open_with_flags(
            file,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        let snapshot = connection.unchecked_transaction()?;
        let last_arrival = last_in(&snapshot, "arrivals")?;
        let last_message_change = last_in(&snapshot, "message_changes")?;
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
        let tasks = TaskOrder::read(&snapshot)?;
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
            last_message_change,
            tasks,
        })
    }

    pub fn is_read_only(&self) -> bool {
        self.connection
            .is_readonly(rusqlite::MAIN_DB)
            .unwrap_or(false)
    }

    pub fn search(&self, query: &str) -> Result<SearchResults, ReplicaError> {
        Ok(search::search(&self.connection, query)?)
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
        let snapshot = self.connection.unchecked_transaction()?;
        let arrived = messages_logged_since(
            &snapshot,
            "arrivals",
            self.last_arrival,
            &mut self.channels,
            &mut self.authors,
        )?;
        let changed = messages_logged_since(
            &snapshot,
            "message_changes",
            self.last_message_change,
            &mut self.channels,
            &mut self.authors,
        )?;
        snapshot.commit()?;
        self.last_arrival = arrived.last;
        self.last_message_change = changed.last;
        let anything_arrived = !arrived.messages.is_empty();
        place_in_order(&mut self.flow, arrived.messages);
        let anything_moved = take_in_changes(&mut self.flow, changed.messages);
        if !(anything_arrived || anything_moved) {
            return Ok(false);
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
        let lines = self.lines_within(channel);
        self.window_of(
            lines.len(),
            |index| lines.get(index).and_then(|line| self.lines.get(*line)),
            start,
            count,
        )
    }

    pub fn flow_lines_after_in(&self, channel: Option<&str>, created_at: &str, id: &str) -> u32 {
        let first_after = self
            .flow
            .partition_point(|message| !message.comes_after(created_at, id));
        let ends_before = |line: &Line| line.latest().is_some_and(|latest| latest < first_after);
        let lines_after = match channel {
            None => self.lines.len() - self.lines.partition_point(ends_before),
            Some(channel) => {
                let lines = self.lines_within(channel);
                let lines_before =
                    lines.partition_point(|line| self.lines.get(*line).is_some_and(ends_before));
                lines.len() - lines_before
            }
        };
        u32::try_from(lines_after).unwrap_or(u32::MAX)
    }

    fn lines_within(&self, channel: &str) -> Ref<'_, [usize]> {
        let is_current = self.narrowed.borrow().as_ref().is_some_and(|narrowed| {
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
            *self.narrowed.borrow_mut() = Some(Narrowed {
                channel: channel.to_owned(),
                revision: self.revision,
                lines,
            });
        }
        Ref::map(self.narrowed.borrow(), |narrowed| {
            narrowed
                .as_ref()
                .map(|narrowed| narrowed.lines.as_slice())
                .unwrap_or_default()
        })
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

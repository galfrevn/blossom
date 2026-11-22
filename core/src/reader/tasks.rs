use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;

use rusqlite::{CachedStatement, Connection, Row, params};

use super::numbers::Numbers;
use super::text_at;
use crate::contract::{Lease, TaskCount, TaskNarrowing, TaskRow, TaskWindow};
use crate::reader::ReplicaReader;
use crate::replica::ReplicaError;

const STATES_IN_THE_ORDER_OF_THE_LENS: [&str; 7] = [
    "blocked",
    "failed",
    "claimed",
    "ready",
    "waiting",
    "completed",
    "canceled",
];

const ORDERED_TASK_COLUMNS: &str = "tasks.id,
     tasks.state,
     tasks.channel,
     tasks.title,
     coalesce(tasks.latest_event_at, messages.created_at, '')";

const ROW_OF_A_TASK: &str = "SELECT tasks.title,
        tasks.channel,
        coalesce(owners.handle, tasks.owner),
        CASE
            WHEN tasks.claim_actor IS NOT NULL THEN coalesce(holders.name, tasks.claim_actor)
            WHEN tasks.state IN ('blocked', 'failed') THEN (
                SELECT coalesce(actors.name, task_events.actor)
                FROM task_events LEFT JOIN actors ON actors.id = task_events.actor
                WHERE task_events.task = tasks.id
                    AND task_events.event_type = 'task.' || tasks.state
                ORDER BY task_events.position DESC
                LIMIT 1
            )
        END,
        tasks.claim_until,
        CASE WHEN tasks.claim_actor IS NOT NULL THEN (
            SELECT created_at FROM task_events
            WHERE task = tasks.id AND event_type IN ('task.claimed', 'task.renewed')
            ORDER BY position DESC
            LIMIT 1
        ) END
 FROM tasks
 LEFT JOIN actors AS owners ON owners.id = tasks.owner
 LEFT JOIN actors AS holders ON holders.id = tasks.claim_actor
 WHERE tasks.id = ?1";

struct OrderedTask {
    id: Box<str>,
    state: Box<str>,
    rank: usize,
    changed_at: Box<str>,
    channel: u32,
    lowercase_title: Box<str>,
}

impl OrderedTask {
    fn comes_before(&self, other: &OrderedTask) -> Ordering {
        self.rank
            .cmp(&other.rank)
            .then_with(|| other.changed_at.cmp(&self.changed_at))
            .then_with(|| self.id.cmp(&other.id))
    }

    fn is_within(&self, channels_within: &[bool]) -> bool {
        usize::try_from(self.channel)
            .ok()
            .and_then(|number| channels_within.get(number).copied())
            .unwrap_or(false)
    }
}

fn rank_of(state: &str) -> usize {
    STATES_IN_THE_ORDER_OF_THE_LENS
        .iter()
        .position(|known| *known == state)
        .unwrap_or(STATES_IN_THE_ORDER_OF_THE_LENS.len())
}

fn ordered_task(
    row: &Row<'_>,
    first_column: usize,
    channels: &mut Numbers,
) -> rusqlite::Result<OrderedTask> {
    let state = text_at(row, first_column + 1)?;
    Ok(OrderedTask {
        id: text_at(row, first_column)?.into(),
        state: state.into(),
        rank: rank_of(state),
        channel: channels.number_of(text_at(row, first_column + 2)?),
        lowercase_title: text_at(row, first_column + 3)?.to_lowercase().into(),
        changed_at: text_at(row, first_column + 4)?.into(),
    })
}

struct Narrowed {
    narrowing: TaskNarrowing,
    revision: u64,
    positions: Vec<usize>,
}

struct Matching<'narrowing> {
    channels_within: Option<Vec<bool>>,
    state: Option<&'narrowing str>,
    lowercase_title: String,
}

impl Matching<'_> {
    fn matches(&self, task: &OrderedTask) -> bool {
        let in_the_channel = self
            .channels_within
            .as_deref()
            .is_none_or(|channels_within| task.is_within(channels_within));
        let in_the_state = self.state.is_none_or(|state| *task.state == *state);
        let titled = task.lowercase_title.contains(&self.lowercase_title);
        in_the_channel && in_the_state && titled
    }
}

fn narrows_nothing(narrowing: &TaskNarrowing) -> bool {
    narrowing.channel.is_none() && narrowing.state.is_none() && narrowing.title.trim().is_empty()
}

#[derive(Default)]
pub(super) struct TaskOrder {
    tasks: Vec<OrderedTask>,
    channels: Numbers,
    last_change: i64,
    revision: u64,
    narrowed: RefCell<Option<Narrowed>>,
}

impl TaskOrder {
    pub(super) fn read(connection: &Connection) -> rusqlite::Result<TaskOrder> {
        let last_change = connection.query_row(
            "SELECT coalesce(max(sequence), 0) FROM task_changes",
            [],
            |row| row.get(0),
        )?;
        let mut order = TaskOrder {
            last_change,
            ..TaskOrder::default()
        };
        let mut statement = connection.prepare(&format!(
            "SELECT {ORDERED_TASK_COLUMNS}
             FROM tasks LEFT JOIN messages ON messages.id = tasks.id"
        ))?;
        let mut rows = statement.query([])?;
        while let Some(row) = rows.next()? {
            let task = ordered_task(row, 0, &mut order.channels)?;
            order.tasks.push(task);
        }
        order.tasks.sort_by(OrderedTask::comes_before);
        Ok(order)
    }

    fn catch_up(&mut self, connection: &Connection) -> rusqlite::Result<bool> {
        let mut changed: HashMap<Box<str>, OrderedTask> = HashMap::new();
        let mut last_change = self.last_change;
        {
            let mut statement = connection.prepare_cached(&format!(
                "SELECT task_changes.sequence, {ORDERED_TASK_COLUMNS}
                 FROM task_changes
                 JOIN tasks ON tasks.id = task_changes.id
                 LEFT JOIN messages ON messages.id = tasks.id
                 WHERE task_changes.sequence > ?1
                 ORDER BY task_changes.sequence"
            ))?;
            let mut rows = statement.query(params![self.last_change])?;
            while let Some(row) = rows.next()? {
                last_change = row.get(0)?;
                let task = ordered_task(row, 1, &mut self.channels)?;
                changed.insert(task.id.clone(), task);
            }
        }
        if changed.is_empty() {
            return Ok(false);
        }
        self.last_change = last_change;
        self.tasks.retain(|task| !changed.contains_key(&task.id));
        self.tasks.extend(changed.into_values());
        self.tasks.sort_by(OrderedTask::comes_before);
        self.revision = self.revision.wrapping_add(1);
        Ok(true)
    }

    fn matching<'narrowing>(&self, narrowing: &'narrowing TaskNarrowing) -> Matching<'narrowing> {
        Matching {
            channels_within: narrowing
                .channel
                .as_deref()
                .map(|channel| self.channels.within(channel)),
            state: narrowing.state.as_deref(),
            lowercase_title: narrowing.title.trim().to_lowercase(),
        }
    }

    fn positions_narrowed_by(&self, narrowing: &TaskNarrowing) -> Vec<usize> {
        let matching = self.matching(narrowing);
        self.tasks
            .iter()
            .enumerate()
            .filter(|(_, task)| matching.matches(task))
            .map(|(position, _)| position)
            .collect()
    }

    fn counts_in(&self, channel: Option<&str>) -> Vec<TaskCount> {
        let narrowing = TaskNarrowing {
            channel: channel.map(str::to_owned),
            ..TaskNarrowing::default()
        };
        let matching = self.matching(&narrowing);
        let mut counts: Vec<TaskCount> = STATES_IN_THE_ORDER_OF_THE_LENS
            .iter()
            .map(|state| TaskCount {
                state: (*state).to_owned(),
                count: 0,
            })
            .collect();
        for task in self.tasks.iter().filter(|task| matching.matches(task)) {
            match counts.iter_mut().find(|count| *count.state == *task.state) {
                Some(count) => count.count = count.count.saturating_add(1),
                None => counts.push(TaskCount {
                    state: task.state.to_string(),
                    count: 1,
                }),
            }
        }
        counts
    }
}

impl ReplicaReader {
    pub fn catch_up_tasks(&mut self) -> Result<bool, ReplicaError> {
        Ok(self.tasks.catch_up(&self.connection)?)
    }

    pub fn task_counts_in(&self, channel: Option<&str>) -> Vec<TaskCount> {
        self.tasks.counts_in(channel)
    }

    pub fn task_window_in(
        &self,
        narrowing: &TaskNarrowing,
        start: u32,
        count: u32,
    ) -> Result<TaskWindow, ReplicaError> {
        let order = &self.tasks;
        if narrows_nothing(narrowing) {
            return self.task_window_of(
                order.tasks.len(),
                |index| order.tasks.get(index),
                start,
                count,
            );
        }
        let mut narrowed = order.narrowed.borrow_mut();
        let is_current = narrowed.as_ref().is_some_and(|narrowed| {
            narrowed.narrowing == *narrowing && narrowed.revision == order.revision
        });
        if !is_current {
            *narrowed = Some(Narrowed {
                narrowing: narrowing.clone(),
                revision: order.revision,
                positions: order.positions_narrowed_by(narrowing),
            });
        }
        let positions = narrowed
            .as_ref()
            .map(|narrowed| narrowed.positions.as_slice())
            .unwrap_or_default();
        self.task_window_of(
            positions.len(),
            |index| {
                positions
                    .get(index)
                    .and_then(|position| order.tasks.get(*position))
            },
            start,
            count,
        )
    }

    fn task_window_of<'order>(
        &'order self,
        total: usize,
        task_at: impl Fn(usize) -> Option<&'order OrderedTask>,
        start: u32,
        count: u32,
    ) -> Result<TaskWindow, ReplicaError> {
        let first = usize::try_from(start).unwrap_or(usize::MAX).min(total);
        let wanted = usize::try_from(count).unwrap_or(usize::MAX);
        let last = first.saturating_add(wanted).min(total);
        let mut statement = self.connection.prepare_cached(ROW_OF_A_TASK)?;
        let mut rows = Vec::with_capacity(last - first);
        for index in first..last {
            let Some(task) = task_at(index) else {
                break;
            };
            rows.push(read_task_row(&mut statement, task)?);
        }
        Ok(TaskWindow {
            total: u32::try_from(total).unwrap_or(u32::MAX),
            start: u32::try_from(first).unwrap_or(u32::MAX),
            rows,
        })
    }
}

fn read_task_row(
    statement: &mut CachedStatement<'_>,
    task: &OrderedTask,
) -> Result<TaskRow, ReplicaError> {
    let row = statement.query_row(params![&*task.id], |row| {
        let lease_until: Option<String> = row.get(4)?;
        let lease_since: Option<String> = row.get(5)?;
        Ok(TaskRow {
            id: task.id.to_string(),
            state: task.state.to_string(),
            title: row.get(0)?,
            channel: row.get(1)?,
            owner: row.get(2)?,
            holder: row.get(3)?,
            lease: lease_until.map(|until| Lease {
                since: lease_since,
                until,
            }),
            changed_at: task.changed_at.to_string(),
        })
    })?;
    Ok(row)
}

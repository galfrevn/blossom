use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};

use rusqlite::{Connection, Transaction, params};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ReadPoint {
    pub(crate) task_changes: i64,
    pub(crate) message_changes: i64,
}

impl ReadPoint {
    const BEFORE_ANY_ROW: ReadPoint = ReadPoint {
        task_changes: 0,
        message_changes: 0,
    };

    fn lowest_with(self, other: ReadPoint) -> ReadPoint {
        ReadPoint {
            task_changes: self.task_changes.min(other.task_changes),
            message_changes: self.message_changes.min(other.message_changes),
        }
    }
}

#[derive(Default)]
struct OpenReaders {
    ever_opened: u64,
    read_so_far: HashMap<u64, (String, ReadPoint)>,
}

fn open_readers() -> MutexGuard<'static, OpenReaders> {
    static OPEN_READERS: OnceLock<Mutex<OpenReaders>> = OnceLock::new();
    OPEN_READERS
        .get_or_init(Mutex::default)
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

fn replica_read_through(connection: &Connection) -> String {
    connection.path().unwrap_or_default().to_owned()
}

pub(crate) struct OpenReader {
    number: u64,
}

impl OpenReader {
    pub(crate) fn of(connection: &Connection) -> OpenReader {
        let replica = replica_read_through(connection);
        let mut readers = open_readers();
        readers.ever_opened = readers.ever_opened.wrapping_add(1);
        let number = readers.ever_opened;
        readers
            .read_so_far
            .insert(number, (replica, ReadPoint::BEFORE_ANY_ROW));
        OpenReader { number }
    }

    pub(crate) fn has_read(&self, point: ReadPoint) {
        if let Some((_, read_so_far)) = open_readers().read_so_far.get_mut(&self.number) {
            *read_so_far = point;
        }
    }
}

impl Drop for OpenReader {
    fn drop(&mut self) {
        open_readers().read_so_far.remove(&self.number);
    }
}

fn read_by_every_open_reader_of(replica: &str) -> Option<ReadPoint> {
    open_readers()
        .read_so_far
        .values()
        .filter(|(read_replica, _)| read_replica == replica)
        .map(|(_, point)| *point)
        .reduce(ReadPoint::lowest_with)
}

pub(super) fn delete_what_every_open_reader_has_read(
    transaction: &Transaction<'_>,
) -> rusqlite::Result<()> {
    let replica = replica_read_through(transaction);
    let Some(read_by_every_reader) = read_by_every_open_reader_of(&replica) else {
        return Ok(());
    };
    transaction
        .prepare_cached("DELETE FROM task_changes WHERE sequence <= ?1")?
        .execute(params![read_by_every_reader.task_changes])?;
    transaction
        .prepare_cached("DELETE FROM message_changes WHERE sequence <= ?1")?
        .execute(params![read_by_every_reader.message_changes])?;
    Ok(())
}

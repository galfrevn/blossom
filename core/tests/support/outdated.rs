use std::path::Path;

use rusqlite::{Connection, params};

pub fn write_a_replica_from_before_task_events(
    file: &Path,
    oldest_id: &str,
    oldest_created_at: &str,
    feed_cursor: &str,
) -> rusqlite::Result<()> {
    let connection = Connection::open(file)?;
    connection.execute_batch(
        "CREATE TABLE messages (
             id TEXT PRIMARY KEY,
             channel TEXT NOT NULL,
             thread TEXT NOT NULL,
             created_at TEXT NOT NULL,
             version INTEGER NOT NULL,
             view TEXT NOT NULL
         ) WITHOUT ROWID;
         CREATE TABLE feed (
             only_row INTEGER PRIMARY KEY CHECK (only_row = 1),
             cursor TEXT NOT NULL
         );
         CREATE TABLE board (
             only_row INTEGER PRIMARY KEY CHECK (only_row = 1),
             oldest_id TEXT NOT NULL,
             oldest_created_at TEXT NOT NULL
         );",
    )?;
    connection.execute(
        "INSERT INTO board (only_row, oldest_id, oldest_created_at) VALUES (1, ?1, ?2)",
        params![oldest_id, oldest_created_at],
    )?;
    connection.execute(
        "INSERT INTO feed (only_row, cursor) VALUES (1, ?1)",
        params![feed_cursor],
    )?;
    Ok(())
}

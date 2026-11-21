use std::path::Path;

use rusqlite::{Connection, OpenFlags};

use crate::replica::ReplicaError;

pub struct ReplicaReader {
    connection: Connection,
}

impl ReplicaReader {
    pub fn open(file: &Path) -> Result<ReplicaReader, ReplicaError> {
        let connection = Connection::open_with_flags(
            file,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        Ok(ReplicaReader { connection })
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
}

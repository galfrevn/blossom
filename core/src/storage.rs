use std::fmt;

use crate::replica::{MessageView, Replica, ReplicaError};

#[derive(Debug)]
pub enum StorageError {
    Replica(ReplicaError),
    Thread(tokio::task::JoinError),
}

impl fmt::Display for StorageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::Replica(error) => write!(formatter, "{error}"),
            StorageError::Thread(error) => {
                write!(formatter, "the replica's storage thread stopped: {error}")
            }
        }
    }
}

impl std::error::Error for StorageError {}

pub(crate) async fn store(
    mut replica: Replica,
    views: Vec<MessageView>,
    feed_cursor: Option<String>,
) -> Result<(Replica, u32), StorageError> {
    tokio::task::spawn_blocking(move || {
        match feed_cursor {
            Some(feed_cursor) => replica.apply_from_feed(&views, &feed_cursor)?,
            None => replica.apply(&views)?,
        }
        let stored = replica.message_count();
        Ok((replica, stored))
    })
    .await
    .map_err(StorageError::Thread)?
    .map_err(StorageError::Replica)
}

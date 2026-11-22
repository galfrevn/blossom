use std::fmt;

use crate::replica::{Batch, BoardIdentity, Replica, ReplicaError};

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
    batch: Batch,
    feed_cursor: Option<String>,
) -> Result<(Replica, u32), StorageError> {
    tokio::task::spawn_blocking(move || {
        match feed_cursor {
            Some(feed_cursor) => replica.apply_from_feed(&batch, &feed_cursor)?,
            None => replica.apply(&batch)?,
        }
        let stored = replica.message_count();
        Ok((replica, stored))
    })
    .await
    .map_err(StorageError::Thread)?
    .map_err(StorageError::Replica)
}

pub(crate) async fn finish_copy(
    mut replica: Replica,
    identity: Option<BoardIdentity>,
    feed_cursor: String,
) -> Result<Replica, StorageError> {
    tokio::task::spawn_blocking(move || {
        replica.finish_copy(identity.as_ref(), &feed_cursor)?;
        Ok(replica)
    })
    .await
    .map_err(StorageError::Thread)?
    .map_err(StorageError::Replica)
}

pub(crate) async fn unseen_actors(
    replica: Replica,
    named_ids: Vec<String>,
) -> Result<(Replica, Vec<String>), StorageError> {
    if named_ids.is_empty() {
        return Ok((replica, Vec::new()));
    }
    tokio::task::spawn_blocking(move || {
        let unseen_ids = replica.unseen_actors(&named_ids)?;
        Ok((replica, unseen_ids))
    })
    .await
    .map_err(StorageError::Thread)?
    .map_err(StorageError::Replica)
}

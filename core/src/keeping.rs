use std::convert::Infallible;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use crate::access::BoardAccess;
use crate::contract::{ReplicaState, StopReason};
use crate::copy::{CopyError, copy_board};
use crate::feed::{FollowError, follow_feed};
use crate::pacing::Pacing;
use crate::replica::Replica;

pub enum KeepingEnded {
    BoardGone,
    Stopped(String),
}

pub async fn keep_in_step(
    access: &BoardAccess,
    replica_file: &Path,
    pacing: Pacing,
    mut report: impl FnMut(ReplicaState),
) -> KeepingEnded {
    let replica = match open_replica(replica_file.to_path_buf()).await {
        Ok(replica) => replica,
        Err(error) => {
            report(ReplicaState::Stopped {
                stored: 0,
                reason: StopReason::UnwritableReplica,
            });
            return KeepingEnded::Stopped(error);
        }
    };
    let last_stored = AtomicU32::new(replica.message_count());
    let mut report_and_remember = |state: ReplicaState| {
        last_stored.store(state.stored(), Ordering::Relaxed);
        report(state);
    };
    let copied = match copy_board(access, replica, pacing, &mut report_and_remember).await {
        Ok(copied) => copied,
        Err(CopyError::BoardGone) => return KeepingEnded::BoardGone,
        Err(error) => {
            report_and_remember(ReplicaState::Stopped {
                stored: last_stored.load(Ordering::Relaxed),
                reason: error.stop_reason(),
            });
            return KeepingEnded::Stopped(error.to_string());
        }
    };
    let followed: Result<Infallible, FollowError> = follow_feed(
        access,
        copied.replica,
        copied.feed_cursor,
        pacing,
        &mut report_and_remember,
    )
    .await;
    match followed {
        Ok(never) => match never {},
        Err(FollowError::BoardGone) => KeepingEnded::BoardGone,
        Err(error) => {
            report_and_remember(ReplicaState::Stopped {
                stored: last_stored.load(Ordering::Relaxed),
                reason: error.stop_reason(),
            });
            KeepingEnded::Stopped(error.to_string())
        }
    }
}

async fn open_replica(file: PathBuf) -> Result<Replica, String> {
    let opening = tokio::task::spawn_blocking(move || {
        if let Some(folder) = file.parent() {
            fs::create_dir_all(folder)
                .map_err(|error| format!("the replica's folder could not be made: {error}"))?;
        }
        Replica::open(&file).map_err(|error| error.to_string())
    });
    opening
        .await
        .map_err(|error| format!("the replica's storage thread stopped: {error}"))?
}

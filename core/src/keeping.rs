use std::convert::Infallible;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use reqwest::{Client, StatusCode, Url};
use serde::Deserialize;

use crate::access::BoardAccess;
use crate::contract::{ReplicaState, StopReason};
use crate::copy::{CopyError, copy_board};
use crate::feed::{FollowError, follow_feed};
use crate::pacing::{Pacing, Retries};
use crate::protocol;
use crate::replica::{BoardIdentity, Replica, ReplicaError, known_replica_files, new_replica_file};

pub enum KeepingEnded {
    BoardGone,
    Stopped(String),
}

pub async fn keep_in_step(
    access: &BoardAccess,
    replicas_folder: &Path,
    pacing: Pacing,
    mut announce_replica: impl FnMut(&Path),
    mut report: impl FnMut(ReplicaState),
) -> KeepingEnded {
    let last_stored = AtomicU32::new(0);
    let mut report_and_remember = |state: ReplicaState| {
        last_stored.store(state.stored(), Ordering::Relaxed);
        report(state);
    };
    let interrupted = keep(
        access,
        replicas_folder,
        pacing,
        &mut announce_replica,
        &mut report_and_remember,
    )
    .await;
    match interrupted {
        Interrupted::BoardGone => KeepingEnded::BoardGone,
        Interrupted::Failed {
            reason,
            description,
        } => {
            report_and_remember(ReplicaState::Stopped {
                stored: last_stored.load(Ordering::Relaxed),
                reason,
            });
            KeepingEnded::Stopped(description)
        }
    }
}

enum Interrupted {
    BoardGone,
    Failed {
        reason: StopReason,
        description: String,
    },
}

impl Interrupted {
    fn unwritable(description: String) -> Interrupted {
        Interrupted::Failed {
            reason: StopReason::UnwritableReplica,
            description,
        }
    }
}

impl From<CopyError> for Interrupted {
    fn from(error: CopyError) -> Interrupted {
        match error {
            CopyError::BoardGone => Interrupted::BoardGone,
            error => Interrupted::Failed {
                reason: error.stop_reason(),
                description: error.to_string(),
            },
        }
    }
}

impl From<FollowError> for Interrupted {
    fn from(error: FollowError) -> Interrupted {
        match error {
            FollowError::BoardGone => Interrupted::BoardGone,
            error => Interrupted::Failed {
                reason: error.stop_reason(),
                description: error.to_string(),
            },
        }
    }
}

struct Start {
    replica: Replica,
    feed_cursor: String,
    file: PathBuf,
}

async fn keep(
    access: &BoardAccess,
    replicas_folder: &Path,
    pacing: Pacing,
    announce_replica: &mut impl FnMut(&Path),
    report: &mut impl FnMut(ReplicaState),
) -> Interrupted {
    loop {
        let known = match start_from_a_known_replica(access, replicas_folder, pacing, report).await
        {
            Ok(known) => known,
            Err(interrupted) => return interrupted,
        };
        let copied_just_now = known.is_none();
        let start = match known {
            Some(start) => {
                announce_replica(&start.file);
                start
            }
            None => {
                let copied = copy_into_a_new_replica(
                    access,
                    replicas_folder,
                    pacing,
                    &mut *announce_replica,
                    &mut *report,
                )
                .await;
                match copied {
                    Ok(start) => start,
                    Err(interrupted) => return interrupted,
                }
            }
        };
        let followed: Result<Infallible, FollowError> = follow_feed(
            access,
            start.replica,
            start.feed_cursor,
            pacing,
            &mut *report,
        )
        .await;
        match followed {
            Ok(never) => match never {},
            Err(FollowError::CursorRefused) if !copied_just_now => {
                if let Err(description) = discard_replica(start.file).await {
                    return Interrupted::unwritable(description);
                }
            }
            Err(error) => return Interrupted::from(error),
        }
    }
}

async fn start_from_a_known_replica(
    access: &BoardAccess,
    replicas_folder: &Path,
    pacing: Pacing,
    report: &mut impl FnMut(ReplicaState),
) -> Result<Option<Start>, Interrupted> {
    let known_files = known_replica_files(replicas_folder, access);
    if known_files.is_empty() {
        return Ok(None);
    }
    let mut checker = IdentityChecker {
        client: protocol::reading_client(pacing.longest_silence).ok_or_else(|| {
            Interrupted::Failed {
                reason: StopReason::UnreadableBoard,
                description: String::from("no http client could be built"),
            }
        })?,
        access,
        retries: Retries::new(pacing, access),
    };
    for file in known_files {
        let opened = open_known_replica(file.clone())
            .await
            .map_err(Interrupted::unwritable)?;
        let KnownReplicaOpening::Recognizable(known) = opened else {
            discard_replica(file)
                .await
                .map_err(Interrupted::unwritable)?;
            continue;
        };
        let stored = known.replica.message_count();
        let holds = checker
            .board_holds(&known.identity, &mut || {
                report(ReplicaState::Reconnecting { stored });
            })
            .await?;
        if holds {
            return Ok(Some(Start {
                replica: known.replica,
                feed_cursor: known.feed_cursor,
                file,
            }));
        }
    }
    Ok(None)
}

async fn copy_into_a_new_replica(
    access: &BoardAccess,
    replicas_folder: &Path,
    pacing: Pacing,
    announce_replica: &mut impl FnMut(&Path),
    report: &mut impl FnMut(ReplicaState),
) -> Result<Start, Interrupted> {
    let file = new_replica_file(replicas_folder, access).map_err(|error| {
        Interrupted::unwritable(format!("a new replica could not be made: {error}"))
    })?;
    let replica = open_new_replica(file.clone())
        .await
        .map_err(Interrupted::unwritable)?;
    let copied = copy_board(access, replica, pacing, &mut *report, || {
        announce_replica(&file);
    })
    .await?;
    Ok(Start {
        replica: copied.replica,
        feed_cursor: copied.feed_cursor,
        file,
    })
}

struct KnownReplica {
    replica: Replica,
    identity: BoardIdentity,
    feed_cursor: String,
}

enum KnownReplicaOpening {
    Recognizable(KnownReplica),
    MadeWithAnotherSchema,
    Unfinished,
    WithNothingToRecognizeItsBoardBy,
}

async fn open_known_replica(file: PathBuf) -> Result<KnownReplicaOpening, String> {
    on_storage_thread(move || {
        let replica = match Replica::open(&file) {
            Ok(replica) => replica,
            Err(ReplicaError::AnotherSchemaVersion(_)) => {
                return Ok(KnownReplicaOpening::MadeWithAnotherSchema);
            }
            Err(error) => return Err(error.to_string()),
        };
        let Some(feed_cursor) = replica.feed_cursor().map_err(|error| error.to_string())? else {
            return Ok(KnownReplicaOpening::Unfinished);
        };
        let Some(identity) = replica.identity().map_err(|error| error.to_string())? else {
            return Ok(KnownReplicaOpening::WithNothingToRecognizeItsBoardBy);
        };
        Ok(KnownReplicaOpening::Recognizable(KnownReplica {
            replica,
            identity,
            feed_cursor,
        }))
    })
    .await
}

async fn open_new_replica(file: PathBuf) -> Result<Replica, String> {
    on_storage_thread(move || Replica::open(&file).map_err(|error| error.to_string())).await
}

async fn discard_replica(file: PathBuf) -> Result<(), String> {
    on_storage_thread(move || {
        remove_replica_files(&file).map_err(|error| {
            format!("a replica that cannot be followed on could not be removed: {error}")
        })
    })
    .await
}

fn remove_replica_files(file: &Path) -> io::Result<()> {
    let mut companions = vec![file.to_path_buf()];
    for suffix in ["-wal", "-shm"] {
        let mut companion = file.as_os_str().to_owned();
        companion.push(suffix);
        companions.push(PathBuf::from(companion));
    }
    for companion in companions {
        match fs::remove_file(&companion) {
            Err(error) if error.kind() != io::ErrorKind::NotFound => return Err(error),
            _ => {}
        }
    }
    Ok(())
}

async fn on_storage_thread<Value: Send + 'static>(
    work: impl FnOnce() -> Result<Value, String> + Send + 'static,
) -> Result<Value, String> {
    tokio::task::spawn_blocking(work)
        .await
        .map_err(|error| format!("the replica's storage thread stopped: {error}"))?
}

struct IdentityChecker<'access> {
    client: Client,
    access: &'access BoardAccess,
    retries: Retries<'access>,
}

#[derive(Deserialize)]
struct ThreadAround {
    message: MessageKeys,
}

#[derive(Deserialize)]
struct MessageKeys {
    id: String,
    #[serde(rename = "createdAt")]
    created_at: String,
}

impl IdentityChecker<'_> {
    async fn board_holds(
        &mut self,
        identity: &BoardIdentity,
        on_failure: &mut impl FnMut(),
    ) -> Result<bool, Interrupted> {
        let mut address = Url::parse(self.access.url()).map_err(|_| Interrupted::Failed {
            reason: StopReason::UnreadableBoard,
            description: format!("the board's address cannot be read: {}", self.access.url()),
        })?;
        if let Ok(mut segments) = address.path_segments_mut() {
            segments.push("messages").push(&identity.oldest_id);
        }
        address.query_pairs_mut().append_pair("limit", "1");
        loop {
            let sent = self
                .client
                .get(address.clone())
                .bearer_auth(self.access.token())
                .send()
                .await;
            match sent {
                Ok(response) if response.status() == StatusCode::NOT_FOUND => return Ok(false),
                Ok(response) if response.status().is_success() => {
                    self.retries.succeeded();
                    let Ok(thread) = response.json::<ThreadAround>().await else {
                        return Ok(false);
                    };
                    return Ok(thread.message.id == identity.oldest_id
                        && thread.message.created_at == identity.oldest_created_at);
                }
                _ => {
                    on_failure();
                    self.retries
                        .wait_after_failure()
                        .await
                        .map_err(|_| Interrupted::BoardGone)?;
                }
            }
        }
    }
}

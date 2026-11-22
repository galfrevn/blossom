use std::convert::Infallible;
use std::fmt;

use reqwest::{Client, Response, StatusCode, Url};
use serde::Deserialize;
use tokio::time::Instant;

use crate::access::BoardAccess;
use crate::contract::{ReplicaState, StopReason};
use crate::pacing::{Pacing, Retries};
use crate::protocol;
use crate::replica::{Batch, Replica};
use crate::sse::{EventStreamReader, Frame};
use crate::storage::{self, StorageError};

#[derive(Debug)]
pub enum FollowError {
    NoHttpClient,
    InvalidEvent(serde_json::Error),
    Storage(StorageError),
    BoardGone,
    CursorRefused,
}

impl fmt::Display for FollowError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FollowError::NoHttpClient => write!(formatter, "no http client could be built"),
            FollowError::InvalidEvent(error) => {
                write!(
                    formatter,
                    "the board's feed sent an event that cannot be read: {error}"
                )
            }
            FollowError::Storage(error) => write!(formatter, "{error}"),
            FollowError::BoardGone => write!(formatter, "the board stopped answering"),
            FollowError::CursorRefused => {
                write!(formatter, "the board refused the replica's cursor")
            }
        }
    }
}

impl std::error::Error for FollowError {}

impl FollowError {
    pub(crate) fn stop_reason(&self) -> StopReason {
        match self {
            FollowError::Storage(_) => StopReason::UnwritableReplica,
            _ => StopReason::UnreadableBoard,
        }
    }
}

pub async fn follow_feed(
    access: &BoardAccess,
    mut replica: Replica,
    from_cursor: String,
    pacing: Pacing,
    mut report: impl FnMut(ReplicaState),
) -> Result<Infallible, FollowError> {
    let client =
        protocol::reading_client(pacing.longest_silence).ok_or(FollowError::NoHttpClient)?;
    let mut stored = replica.message_count();
    let mut cursor = from_cursor;
    let mut retries = Retries::new(pacing, access);
    loop {
        let opening = open_stream(&client, access, &cursor);
        let opened = tokio::time::timeout(pacing.longest_wait_for_a_stream, opening).await;
        if let Ok(Opening::CursorRefused) = opened {
            return Err(FollowError::CursorRefused);
        }
        let Ok(Opening::Opened(mut stream)) = opened else {
            report(ReplicaState::Reconnecting { stored });
            retries
                .wait_after_failure()
                .await
                .map_err(|_| FollowError::BoardGone)?;
            continue;
        };
        report(ReplicaState::Live { stored });
        let opened_at = Instant::now();
        let cursor_when_opened = cursor.clone();
        let mut reader = EventStreamReader::default();
        let mut stream_failed = false;
        loop {
            let bytes = match stream.chunk().await {
                Ok(Some(bytes)) => bytes,
                Ok(None) => break,
                Err(_) => {
                    stream_failed = true;
                    break;
                }
            };
            let frames = reader.read(&bytes);
            if frames.is_empty() {
                continue;
            }
            let batch = batch_in(&frames)?;
            let newest_id = frames.iter().rev().find_map(|frame| frame.id.clone());
            let cursor_moved = newest_id.as_ref().is_some_and(|id| *id != cursor);
            if batch.is_empty() && !cursor_moved {
                continue;
            }
            let answered_batch;
            (replica, answered_batch) =
                with_unseen_actors_asked_for(&client, access, replica, batch).await?;
            let Some(batch) = answered_batch else {
                stream_failed = true;
                break;
            };
            if let Some(newest_id) = newest_id {
                cursor = newest_id;
            }
            (replica, stored) = storage::store(replica, batch, Some(cursor.clone()))
                .await
                .map_err(FollowError::Storage)?;
            report(ReplicaState::Live { stored });
        }
        let ended_at_once_without_progress =
            cursor == cursor_when_opened && opened_at.elapsed() < pacing.first_pause;
        if stream_failed || ended_at_once_without_progress {
            report(ReplicaState::Reconnecting { stored });
            retries
                .wait_after_failure()
                .await
                .map_err(|_| FollowError::BoardGone)?;
        } else {
            retries.succeeded();
        }
    }
}

enum Opening {
    Opened(Response),
    CursorRefused,
    Failed,
}

#[derive(Deserialize)]
struct BoardRefusal {
    error: RefusalReason,
}

#[derive(Deserialize)]
struct RefusalReason {
    code: String,
}

async fn open_stream(client: &Client, access: &BoardAccess, cursor: &str) -> Opening {
    let Ok(mut address) = Url::parse(&format!("{}/events", access.url())) else {
        return Opening::Failed;
    };
    let (key, value) = protocol::WITH_BOOKKEEPING;
    address.query_pairs_mut().append_pair(key, value);
    let sent = client
        .get(address)
        .bearer_auth(access.token())
        .header("accept", "text/event-stream")
        .header("last-event-id", cursor)
        .send()
        .await;
    let Ok(response) = sent else {
        return Opening::Failed;
    };
    if response.status().is_success() {
        return Opening::Opened(response);
    }
    if response.status() != StatusCode::BAD_REQUEST {
        return Opening::Failed;
    }
    match response.json::<BoardRefusal>().await {
        Ok(refusal) if refusal.error.code == "invalid_cursor" => Opening::CursorRefused,
        _ => Opening::Failed,
    }
}

fn batch_in(frames: &[Frame]) -> Result<Batch, FollowError> {
    let mut batch = Batch::default();
    for data in frames.iter().filter_map(|frame| frame.data.as_deref()) {
        batch.add_event(data).map_err(FollowError::InvalidEvent)?;
    }
    Ok(batch)
}

async fn with_unseen_actors_asked_for(
    client: &Client,
    access: &BoardAccess,
    replica: Replica,
    mut batch: Batch,
) -> Result<(Replica, Option<Batch>), FollowError> {
    let named_ids = batch.actor_ids_without_details();
    let (replica, unseen_ids) = storage::unseen_actors(replica, named_ids)
        .await
        .map_err(FollowError::Storage)?;
    for asked_ids in unseen_ids.chunks(protocol::ACTORS_PER_REQUEST) {
        let Some(listed) = protocol::listed_actors(client, access, asked_ids).await else {
            return Ok((replica, None));
        };
        batch.add_listed_actors(listed, asked_ids);
    }
    Ok((replica, Some(batch)))
}

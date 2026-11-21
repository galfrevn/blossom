use std::fmt;
use std::time::Duration;

use reqwest::{Client, Response};
use tokio::time::Instant;

use crate::access::BoardAccess;
use crate::contract::ReplicaState;
use crate::pacing::Pacing;
use crate::protocol::{self, FeedEvent};
use crate::replica::{MessageView, Replica};
use crate::sse::{EventStreamReader, Frame};
use crate::storage::{self, StorageError};

#[derive(Debug)]
pub enum FollowError {
    NoHttpClient,
    InvalidEvent(serde_json::Error),
    Storage(StorageError),
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
        }
    }
}

impl std::error::Error for FollowError {}

pub async fn follow_feed(
    access: &BoardAccess,
    mut replica: Replica,
    from_cursor: String,
    pacing: Pacing,
    mut report: impl FnMut(ReplicaState),
) -> Result<(), FollowError> {
    let client = protocol::reading_client().ok_or(FollowError::NoHttpClient)?;
    let mut stored = replica.message_count();
    let mut cursor = from_cursor;
    let mut pause = pacing.first_pause();
    loop {
        let Some(mut stream) = open_stream(&client, access, &cursor).await else {
            report(ReplicaState::Reconnecting { stored });
            pause = wait_before_reconnecting(pause, &pacing).await;
            continue;
        };
        report(ReplicaState::Live { stored });
        let opened_at = Instant::now();
        let cursor_when_opened = cursor.clone();
        let mut reader = EventStreamReader::default();
        while let Ok(Some(bytes)) = stream.chunk().await {
            let frames = reader.read(&bytes);
            if frames.is_empty() {
                continue;
            }
            let views = views_in(&frames)?;
            let newest_id = frames.iter().rev().find_map(|frame| frame.id.clone());
            let cursor_moved = newest_id.as_ref().is_some_and(|id| *id != cursor);
            if views.is_empty() && !cursor_moved {
                continue;
            }
            if let Some(newest_id) = newest_id {
                cursor = newest_id;
            }
            (replica, stored) = storage::store(replica, views, Some(cursor.clone()))
                .await
                .map_err(FollowError::Storage)?;
            report(ReplicaState::Live { stored });
        }
        let ended_at_once_without_progress =
            cursor == cursor_when_opened && opened_at.elapsed() < pacing.first_pause();
        if ended_at_once_without_progress {
            report(ReplicaState::Reconnecting { stored });
            pause = wait_before_reconnecting(pause, &pacing).await;
        } else {
            pause = pacing.first_pause();
        }
    }
}

async fn wait_before_reconnecting(pause: Duration, pacing: &Pacing) -> Duration {
    tokio::time::sleep(pause).await;
    pacing.pause_after(pause)
}

async fn open_stream(client: &Client, access: &BoardAccess, cursor: &str) -> Option<Response> {
    let response = client
        .get(format!("{}/events", access.url()))
        .bearer_auth(access.token())
        .header("accept", "text/event-stream")
        .header("last-event-id", cursor)
        .send()
        .await
        .ok()?;
    response.error_for_status().ok()
}

fn views_in(frames: &[Frame]) -> Result<Vec<MessageView>, FollowError> {
    let mut views = Vec::new();
    for data in frames.iter().filter_map(|frame| frame.data.as_deref()) {
        let event: FeedEvent = serde_json::from_str(data).map_err(FollowError::InvalidEvent)?;
        if let Some(message) = event.message {
            views.push(MessageView::parse(message.get()).map_err(FollowError::InvalidEvent)?);
        }
    }
    Ok(views)
}

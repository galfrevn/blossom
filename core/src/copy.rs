use std::fmt;

use reqwest::{Client, Url};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::value::RawValue;

use crate::access::BoardAccess;
use crate::contract::{ReplicaState, StopReason};
use crate::pacing::{Pacing, Retries};
use crate::protocol::{self, FeedEvent};
use crate::replica::{BoardIdentity, MessageView, Replica};
use crate::storage::{self, StorageError};

const THREADS_PER_PAGE: &str = "500";
const CHANNELS_PER_PAGE: &str = "500";

pub struct CopiedBoard {
    pub replica: Replica,
    pub feed_cursor: String,
    pub identity: Option<BoardIdentity>,
}

#[derive(Debug)]
pub enum CopyError {
    NoHttpClient,
    InvalidAddress(String),
    Request(reqwest::Error),
    InvalidView(serde_json::Error),
    Storage(StorageError),
    BoardGone,
}

impl fmt::Display for CopyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CopyError::NoHttpClient => write!(formatter, "no http client could be built"),
            CopyError::InvalidAddress(address) => write!(formatter, "{address} is not a url"),
            CopyError::Request(error) => {
                write!(formatter, "a request to the board failed: {error}")
            }
            CopyError::InvalidView(error) => {
                write!(
                    formatter,
                    "the board sent a message view the copy cannot read: {error}"
                )
            }
            CopyError::Storage(error) => write!(formatter, "{error}"),
            CopyError::BoardGone => write!(formatter, "the board stopped answering"),
        }
    }
}

impl std::error::Error for CopyError {}

impl CopyError {
    pub(crate) fn stop_reason(&self) -> StopReason {
        match self {
            CopyError::Storage(_) => StopReason::UnwritableReplica,
            _ => StopReason::UnreadableBoard,
        }
    }
}

#[derive(Deserialize)]
struct ThreadsPage {
    threads: Vec<ThreadSummary>,
    cursor: String,
    older: Option<String>,
}

#[derive(Deserialize)]
struct ThreadSummary {
    message: Box<RawValue>,
    last: Option<Box<RawValue>>,
}

#[derive(Deserialize)]
struct ChannelsPage {
    channels: Vec<ChannelSummary>,
    next: Option<String>,
}

#[derive(Deserialize)]
struct ChannelSummary {
    messages: Option<u64>,
}

#[derive(Deserialize)]
struct EventsPage {
    events: Vec<FeedEvent>,
    cursor: String,
}

pub async fn copy_board(
    access: &BoardAccess,
    mut replica: Replica,
    pacing: Pacing,
    mut report: impl FnMut(ReplicaState),
) -> Result<CopiedBoard, CopyError> {
    let mut reader = PatientReader {
        client: protocol::reading_client(pacing.longest_silence).ok_or(CopyError::NoHttpClient)?,
        access,
        retries: Retries::new(pacing, access),
    };
    let mut stored = replica.message_count();
    let total = message_total(&mut reader, &mut || {
        report(ReplicaState::Reconnecting { stored });
    })
    .await?;

    let threads = read_every_thread(&mut reader, &mut || {
        report(ReplicaState::Reconnecting { stored });
    })
    .await?;
    let identity = threads.oldest_root;
    let mut feed_cursor = threads.feed_cursor;
    (replica, stored) = storage::store(replica, threads.views, None)
        .await
        .map_err(CopyError::Storage)?;
    report(ReplicaState::Copying { stored, total });

    if let Some(oldest_root) = identity.as_ref() {
        let mut after = oldest_root.oldest_id.clone();
        loop {
            let page: EventsPage = reader
                .read("/events", &[("after", &after)], &mut || {
                    report(ReplicaState::Reconnecting { stored });
                })
                .await?;
            let views = page
                .events
                .iter()
                .filter_map(|event| event.message.as_deref())
                .map(|message| MessageView::parse(message.get()))
                .collect::<Result<Vec<_>, _>>()
                .map_err(CopyError::InvalidView)?;
            (replica, stored) = storage::store(replica, views, None)
                .await
                .map_err(CopyError::Storage)?;
            report(ReplicaState::Copying { stored, total });
            if page.cursor == after {
                break;
            }
            after = page.cursor;
        }
        feed_cursor = after;
    }

    let (replica, _) = storage::store(replica, Vec::new(), Some(feed_cursor.clone()))
        .await
        .map_err(CopyError::Storage)?;
    Ok(CopiedBoard {
        replica,
        feed_cursor,
        identity,
    })
}

struct EveryThread {
    views: Vec<MessageView>,
    oldest_root: Option<BoardIdentity>,
    feed_cursor: String,
}

async fn read_every_thread(
    reader: &mut PatientReader<'_>,
    on_failure: &mut impl FnMut(),
) -> Result<EveryThread, CopyError> {
    let mut views = Vec::new();
    let mut oldest_root: Option<BoardIdentity> = None;
    let mut feed_cursor = None;
    let mut before: Option<String> = None;
    loop {
        let mut query = vec![("limit", THREADS_PER_PAGE)];
        if let Some(before) = before.as_deref() {
            query.push(("before", before));
        }
        let page: ThreadsPage = reader.read("/threads", &query, on_failure).await?;
        feed_cursor.get_or_insert(page.cursor);
        for thread in page.threads {
            let root = MessageView::parse(thread.message.get()).map_err(CopyError::InvalidView)?;
            let root_is_the_oldest_yet = oldest_root
                .as_ref()
                .is_none_or(|oldest| root.created_at() < oldest.oldest_created_at.as_str());
            if root_is_the_oldest_yet {
                oldest_root = Some(BoardIdentity {
                    oldest_id: root.id().to_owned(),
                    oldest_created_at: root.created_at().to_owned(),
                });
            }
            views.push(root);
            if let Some(last) = thread.last {
                views.push(MessageView::parse(last.get()).map_err(CopyError::InvalidView)?);
            }
        }
        match page.older {
            Some(older) => before = Some(older),
            None => break,
        }
    }
    Ok(EveryThread {
        views,
        oldest_root,
        feed_cursor: feed_cursor.unwrap_or_default(),
    })
}

async fn message_total(
    reader: &mut PatientReader<'_>,
    on_failure: &mut impl FnMut(),
) -> Result<Option<u32>, CopyError> {
    let mut total: u64 = 0;
    let mut after: Option<String> = None;
    loop {
        let mut query = vec![("limit", CHANNELS_PER_PAGE)];
        if let Some(after) = after.as_deref() {
            query.push(("after", after));
        }
        let page: ChannelsPage = reader.read("/channels", &query, on_failure).await?;
        for channel in page.channels {
            let Some(messages) = channel.messages else {
                return Ok(None);
            };
            total += messages;
        }
        match page.next {
            Some(next) => after = Some(next),
            None => break,
        }
    }
    Ok(u32::try_from(total).ok())
}

struct PatientReader<'access> {
    client: Client,
    access: &'access BoardAccess,
    retries: Retries<'access>,
}

impl PatientReader<'_> {
    async fn read<Page: DeserializeOwned>(
        &mut self,
        path: &str,
        query: &[(&str, &str)],
        on_failure: &mut impl FnMut(),
    ) -> Result<Page, CopyError> {
        let address_text = format!("{}{path}", self.access.url());
        let mut address = Url::parse(&address_text)
            .map_err(|_| CopyError::InvalidAddress(address_text.clone()))?;
        address.query_pairs_mut().extend_pairs(query);
        loop {
            match self.read_once(address.clone()).await {
                Ok(page) => {
                    self.retries.succeeded();
                    return Ok(page);
                }
                Err(error) if error.is_decode() => return Err(CopyError::Request(error)),
                Err(_) => {
                    on_failure();
                    self.retries
                        .wait_after_failure()
                        .await
                        .map_err(|_| CopyError::BoardGone)?;
                }
            }
        }
    }

    async fn read_once<Page: DeserializeOwned>(&self, address: Url) -> reqwest::Result<Page> {
        let response = self
            .client
            .get(address)
            .bearer_auth(self.access.token())
            .send()
            .await?;
        response.error_for_status()?.json().await
    }
}

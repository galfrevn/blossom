use std::fmt;

use reqwest::{Client, Url};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::value::RawValue;

use crate::access::BoardAccess;
use crate::contract::{ReplicaState, StopReason};
use crate::pacing::{Pacing, Retries};
use crate::protocol::{self, ListedActors};
use crate::replica::{Batch, BoardIdentity, MessageView, Replica};
use crate::storage::{self, StorageError};

const NEWEST_THREADS: &str = "25";
const THREADS_PER_PAGE: &str = "500";
const REPLIES_PER_THREAD: &str = "500";
const CHANNELS_PER_PAGE: &str = "500";

pub struct CopiedBoard {
    pub replica: Replica,
    pub feed_cursor: String,
}

#[derive(Debug)]
pub enum CopyError {
    NoHttpClient,
    InvalidAddress(String),
    Request(reqwest::Error),
    InvalidView(serde_json::Error),
    InvalidEvent(serde_json::Error),
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
            CopyError::InvalidEvent(error) => {
                write!(
                    formatter,
                    "the board sent an event the copy cannot read: {error}"
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
struct ThreadWithReplies {
    message: Box<RawValue>,
    replies: Vec<Box<RawValue>>,
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
    events: Vec<Box<RawValue>>,
    cursor: String,
}

pub async fn copy_board(
    access: &BoardAccess,
    mut replica: Replica,
    pacing: Pacing,
    mut report: impl FnMut(ReplicaState),
    mut newest_threads_stored: impl FnMut(),
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

    let mut oldest_root = OldestRoot::default();
    let newest = read_the_newest_threads(&mut reader, &mut oldest_root, &mut || {
        report(ReplicaState::Reconnecting { stored });
    })
    .await?;
    let mut feed_cursor = newest.feed_cursor;
    (replica, stored) = store_with_the_actors_it_names(
        &mut reader,
        replica,
        Batch::from_views(newest.views),
        &mut || {
            report(ReplicaState::Reconnecting { stored });
        },
    )
    .await?;
    report(ReplicaState::Copying { stored, total });
    newest_threads_stored();

    if let Some(older) = newest.older {
        let older_views = read_the_older_threads(&mut reader, older, &mut oldest_root, &mut || {
            report(ReplicaState::Reconnecting { stored });
        })
        .await?;
        (replica, stored) = store_with_the_actors_it_names(
            &mut reader,
            replica,
            Batch::from_views(older_views),
            &mut || {
                report(ReplicaState::Reconnecting { stored });
            },
        )
        .await?;
        report(ReplicaState::Copying { stored, total });
    }

    let identity = oldest_root.identity;
    if let Some(oldest_root) = identity.as_ref() {
        let mut after = oldest_root.oldest_id.clone();
        loop {
            let page: EventsPage = reader
                .read(&["events"], &[("after", &after)], &mut || {
                    report(ReplicaState::Reconnecting { stored });
                })
                .await?;
            let mut events = Batch::default();
            for event in &page.events {
                events
                    .add_event(event.get())
                    .map_err(CopyError::InvalidEvent)?;
            }
            (replica, stored) =
                store_with_the_actors_it_names(&mut reader, replica, events, &mut || {
                    report(ReplicaState::Reconnecting { stored });
                })
                .await?;
            report(ReplicaState::Copying { stored, total });
            if page.cursor == after {
                break;
            }
            after = page.cursor;
        }
        feed_cursor = after;
    }

    let replica = storage::finish_copy(replica, identity, feed_cursor.clone())
        .await
        .map_err(CopyError::Storage)?;
    Ok(CopiedBoard {
        replica,
        feed_cursor,
    })
}

async fn store_with_the_actors_it_names(
    reader: &mut PatientReader<'_>,
    replica: Replica,
    mut batch: Batch,
    on_failure: &mut impl FnMut(),
) -> Result<(Replica, u32), CopyError> {
    let named_ids = batch.actor_ids_without_details();
    let (replica, unseen_ids) = storage::unseen_actors(replica, named_ids)
        .await
        .map_err(CopyError::Storage)?;
    for asked_ids in unseen_ids.chunks(protocol::ACTORS_PER_REQUEST) {
        let listed: ListedActors = reader
            .read(&["actors"], &protocol::actors_query(asked_ids), on_failure)
            .await?;
        batch.add_listed_actors(listed.actors, asked_ids);
    }
    storage::store(replica, batch, None)
        .await
        .map_err(CopyError::Storage)
}

#[derive(Default)]
struct OldestRoot {
    identity: Option<BoardIdentity>,
}

impl OldestRoot {
    fn consider(&mut self, root: &MessageView) {
        let root_is_the_oldest_yet = self
            .identity
            .as_ref()
            .is_none_or(|oldest| root.created_at() < oldest.oldest_created_at.as_str());
        if root_is_the_oldest_yet {
            self.identity = Some(BoardIdentity {
                oldest_id: root.id().to_owned(),
                oldest_created_at: root.created_at().to_owned(),
            });
        }
    }
}

struct NewestThreads {
    views: Vec<MessageView>,
    feed_cursor: String,
    older: Option<String>,
}

async fn read_the_newest_threads(
    reader: &mut PatientReader<'_>,
    oldest_root: &mut OldestRoot,
    on_failure: &mut impl FnMut(),
) -> Result<NewestThreads, CopyError> {
    let page: ThreadsPage = reader
        .read(&["threads"], &[("limit", NEWEST_THREADS)], on_failure)
        .await?;
    let mut views = Vec::new();
    for thread in page.threads {
        let listed_root =
            MessageView::parse(thread.message.get()).map_err(CopyError::InvalidView)?;
        oldest_root.consider(&listed_root);
        let read: ThreadWithReplies = reader
            .read(
                &["messages", listed_root.id()],
                &[("limit", REPLIES_PER_THREAD)],
                on_failure,
            )
            .await?;
        views.push(MessageView::parse(read.message.get()).map_err(CopyError::InvalidView)?);
        for reply in read.replies {
            views.push(MessageView::parse(reply.get()).map_err(CopyError::InvalidView)?);
        }
    }
    Ok(NewestThreads {
        views,
        feed_cursor: page.cursor,
        older: page.older,
    })
}

async fn read_the_older_threads(
    reader: &mut PatientReader<'_>,
    older: String,
    oldest_root: &mut OldestRoot,
    on_failure: &mut impl FnMut(),
) -> Result<Vec<MessageView>, CopyError> {
    let mut views = Vec::new();
    let mut before = older;
    loop {
        let query = [("limit", THREADS_PER_PAGE), ("before", before.as_str())];
        let page: ThreadsPage = reader.read(&["threads"], &query, on_failure).await?;
        for thread in page.threads {
            let root = MessageView::parse(thread.message.get()).map_err(CopyError::InvalidView)?;
            oldest_root.consider(&root);
            views.push(root);
            if let Some(last) = thread.last {
                views.push(MessageView::parse(last.get()).map_err(CopyError::InvalidView)?);
            }
        }
        match page.older {
            Some(older) => before = older,
            None => return Ok(views),
        }
    }
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
        let page: ChannelsPage = reader.read(&["channels"], &query, on_failure).await?;
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
        path: &[&str],
        query: &[(&str, &str)],
        on_failure: &mut impl FnMut(),
    ) -> Result<Page, CopyError> {
        let address = self.address_of(path, query)?;
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

    fn address_of(&self, path: &[&str], query: &[(&str, &str)]) -> Result<Url, CopyError> {
        let invalid = || CopyError::InvalidAddress(self.access.url().to_owned());
        let mut address = Url::parse(self.access.url()).map_err(|_| invalid())?;
        address
            .path_segments_mut()
            .map_err(|()| invalid())?
            .pop_if_empty()
            .extend(path);
        address.query_pairs_mut().extend_pairs(query);
        Ok(address)
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

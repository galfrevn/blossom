use std::fmt;

use reqwest::{Client, Url};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::value::RawValue;

use crate::access::BoardAccess;
use crate::contract::ReplicaState;
use crate::protocol::{self, FeedEvent};
use crate::replica::{MessageView, Replica};
use crate::storage::{self, StorageError};

const THREADS_PER_PAGE: &str = "500";
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
    Storage(StorageError),
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
        }
    }
}

impl std::error::Error for CopyError {}

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
    replica: Replica,
    mut report: impl FnMut(ReplicaState),
) -> Result<CopiedBoard, CopyError> {
    let client = protocol::client().ok_or(CopyError::NoHttpClient)?;
    let total = message_total(&client, access).await?;

    let threads = read_every_thread(&client, access).await?;
    let oldest_root_id = threads.oldest_root_id;
    let mut feed_cursor = threads.feed_cursor;
    let (mut replica, mut stored) = storage::store(replica, threads.views, None)
        .await
        .map_err(CopyError::Storage)?;
    report(ReplicaState::Copying { stored, total });

    if let Some(oldest_root_id) = oldest_root_id {
        let mut after = oldest_root_id;
        loop {
            let page: EventsPage = read(&client, access, "/events", &[("after", &after)]).await?;
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
    })
}

struct EveryThread {
    views: Vec<MessageView>,
    oldest_root_id: Option<String>,
    feed_cursor: String,
}

async fn read_every_thread(
    client: &Client,
    access: &BoardAccess,
) -> Result<EveryThread, CopyError> {
    let mut views = Vec::new();
    let mut oldest_root: Option<(String, String)> = None;
    let mut feed_cursor = None;
    let mut before: Option<String> = None;
    loop {
        let mut query = vec![("limit", THREADS_PER_PAGE)];
        if let Some(before) = before.as_deref() {
            query.push(("before", before));
        }
        let page: ThreadsPage = read(client, access, "/threads", &query).await?;
        feed_cursor.get_or_insert(page.cursor);
        for thread in page.threads {
            let root = MessageView::parse(thread.message.get()).map_err(CopyError::InvalidView)?;
            let root_is_the_oldest_yet = oldest_root
                .as_ref()
                .is_none_or(|(created_at, _)| root.created_at() < created_at.as_str());
            if root_is_the_oldest_yet {
                oldest_root = Some((root.created_at().to_owned(), root.id().to_owned()));
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
        oldest_root_id: oldest_root.map(|(_, id)| id),
        feed_cursor: feed_cursor.unwrap_or_default(),
    })
}

async fn message_total(client: &Client, access: &BoardAccess) -> Result<Option<u32>, CopyError> {
    let mut total: u64 = 0;
    let mut after: Option<String> = None;
    loop {
        let mut query = vec![("limit", CHANNELS_PER_PAGE)];
        if let Some(after) = after.as_deref() {
            query.push(("after", after));
        }
        let page: ChannelsPage = read(client, access, "/channels", &query).await?;
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

async fn read<Page: DeserializeOwned>(
    client: &Client,
    access: &BoardAccess,
    path: &str,
    query: &[(&str, &str)],
) -> Result<Page, CopyError> {
    let address_text = format!("{}{path}", access.url());
    let mut address =
        Url::parse(&address_text).map_err(|_| CopyError::InvalidAddress(address_text.clone()))?;
    address.query_pairs_mut().extend_pairs(query);
    let response = client
        .get(address)
        .bearer_auth(access.token())
        .send()
        .await
        .map_err(CopyError::Request)?;
    let accepted_response = response.error_for_status().map_err(CopyError::Request)?;
    accepted_response.json().await.map_err(CopyError::Request)
}

#[path = "support/history.rs"]
mod history;

use std::error::Error;
use std::path::PathBuf;
use std::time::Duration;

use blossom::contract::ReplicaState;
use blossom::replica::Replica;
use blossom::{BoardAccess, CopiedBoard, FollowError, Pacing, copy_board, follow_feed};
use history::{BoardHistory, FakeBoard, serve_board};
use tempfile::TempDir;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};
use tokio::task::JoinHandle;
use wiremock::MockServer;

fn history_across_two_channels() -> BoardHistory {
    let mut history = BoardHistory::default();
    history.post("10", "fleet/www", "10", 0, "deploy www");
    history.post("11", "fleet/api", "11", 1, "deploy api");
    history.post("12", "fleet/www", "10", 2, "looking");
    history.post("13", "fleet/www", "13", 3, "flaky test");
    history.post("14", "fleet/api", "11", 4, "done");
    history.post("15", "fleet/www", "13", 5, "rerun");
    history.post("16", "fleet/www", "10", 6, "rolled back");
    history
}

struct Copy {
    _folder: TempDir,
    copied: CopiedBoard,
    reported_states: Vec<ReplicaState>,
}

async fn copy_of(history: &BoardHistory) -> Result<Copy, Box<dyn Error>> {
    let server = MockServer::start().await;
    serve_board(&server, FakeBoard::with_history(history.clone())).await;
    let access = BoardAccess::new(format!("{}/api/board", server.uri()), "token-of-ada");
    let folder = TempDir::new()?;
    let replica = Replica::open(&folder.path().join("board.sqlite"))?;
    let mut reported_states = Vec::new();

    let copied = copy_board(&access, replica, |state| reported_states.push(state)).await?;

    Ok(Copy {
        _folder: folder,
        copied,
        reported_states,
    })
}

#[tokio::test]
async fn a_whole_board_is_copied_into_its_replica() {
    let history = history_across_two_channels();

    let copy = copy_of(&history).await.unwrap();

    assert_eq!(copy.copied.replica.message_count(), 7);
    for id in ["10", "11", "12", "13", "14", "15", "16"] {
        assert_eq!(
            copy.copied.replica.stored_view(id).unwrap().unwrap(),
            history.current_view(id)
        );
    }
}

#[tokio::test]
async fn the_copy_reports_its_progress_against_the_total_and_ends_with_everything_stored() {
    let history = history_across_two_channels();

    let copy = copy_of(&history).await.unwrap();

    assert!(copy.reported_states.contains(&ReplicaState::Copying {
        stored: 7,
        total: Some(7)
    }));
    assert_eq!(
        copy.reported_states.last(),
        Some(&ReplicaState::Copying {
            stored: 7,
            total: Some(7)
        })
    );
}

#[tokio::test]
async fn an_edit_in_the_feed_is_copied_as_it_stands_now() {
    let mut history = history_across_two_channels();
    history.edit("12", "looking, it was the cache");

    let copy = copy_of(&history).await.unwrap();

    assert_eq!(
        copy.copied.replica.stored_view("12").unwrap().unwrap()["text"],
        "looking, it was the cache"
    );
}

#[tokio::test]
async fn a_page_of_the_feed_with_no_messages_does_not_end_the_copy() {
    let mut history = BoardHistory::default();
    history.post("10", "fleet/www", "10", 0, "deploy www");
    history.bookkeeping();
    history.bookkeeping();
    history.bookkeeping();
    history.bookkeeping();
    history.post("11", "fleet/www", "10", 1, "looking");

    let copy = copy_of(&history).await.unwrap();

    assert_eq!(copy.copied.replica.message_count(), 2);
}

#[tokio::test]
async fn an_empty_board_is_copied_as_an_empty_replica() {
    let history = BoardHistory::default();

    let copy = copy_of(&history).await.unwrap();

    assert_eq!(copy.copied.replica.message_count(), 0);
    assert_eq!(
        copy.reported_states.last(),
        Some(&ReplicaState::Copying {
            stored: 0,
            total: Some(0)
        })
    );
}

struct Following {
    _server: MockServer,
    _folder: TempDir,
    file: PathBuf,
    states: UnboundedReceiver<ReplicaState>,
    task: JoinHandle<Result<(), FollowError>>,
}

async fn copy_then_follow(board: &FakeBoard) -> Result<Following, Box<dyn Error>> {
    let quick_pacing = Pacing::new(Duration::from_millis(1), Duration::from_millis(5));
    copy_then_follow_at(board, quick_pacing).await
}

async fn copy_then_follow_at(
    board: &FakeBoard,
    pacing: Pacing,
) -> Result<Following, Box<dyn Error>> {
    let server = MockServer::start().await;
    serve_board(&server, board.clone()).await;
    let access = BoardAccess::new(format!("{}/api/board", server.uri()), "token-of-ada");
    let folder = TempDir::new()?;
    let file = folder.path().join("board.sqlite");
    let replica = Replica::open(&file)?;
    let copied = copy_board(&access, replica, |_| {}).await?;
    let (sender, states) = unbounded_channel();
    let task = tokio::spawn(async move {
        follow_feed(
            &access,
            copied.replica,
            copied.feed_cursor,
            pacing,
            |state| {
                sender.send(state).ok();
            },
        )
        .await
    });
    Ok(Following {
        _server: server,
        _folder: folder,
        file,
        states,
        task,
    })
}

async fn until_reported(
    states: &mut UnboundedReceiver<ReplicaState>,
    expected: &ReplicaState,
) -> bool {
    let waiting = async {
        while let Some(state) = states.recv().await {
            if &state == expected {
                return true;
            }
        }
        false
    };
    tokio::time::timeout(Duration::from_secs(5), waiting)
        .await
        .unwrap_or(false)
}

#[tokio::test]
async fn a_message_posted_while_followed_reaches_the_replica() {
    let board = FakeBoard::with_history(history_across_two_channels());
    let mut following = copy_then_follow(&board).await.unwrap();

    board.change_history(|history| history.post("17", "fleet/www", "13", 7, "green again"));

    assert!(until_reported(&mut following.states, &ReplicaState::Live { stored: 8 }).await);
    following.task.abort();
    let replica = Replica::open(&following.file).unwrap();
    assert_eq!(
        replica.stored_view("17").unwrap().unwrap()["text"],
        "green again"
    );
}

#[tokio::test]
async fn the_feed_is_followed_across_the_board_closing_its_stream() {
    let board = FakeBoard::with_history(history_across_two_channels());
    let mut following = copy_then_follow(&board).await.unwrap();

    board.change_history(|history| {
        for (index, id) in ["17", "18", "19", "20", "21"].iter().enumerate() {
            let minute = 7 + u32::try_from(index).unwrap();
            history.post(id, "fleet/api", "11", minute, "another step");
        }
    });

    assert!(until_reported(&mut following.states, &ReplicaState::Live { stored: 12 }).await);
    following.task.abort();
    let streams_opened = board.stream_starts.lock().unwrap().len();
    let streamed_positions = board.streamed_positions.lock().unwrap().clone();
    assert!(streams_opened >= 3);
    assert_eq!(streamed_positions.len(), 5);
    assert!(streamed_positions.windows(2).all(|pair| pair[0] < pair[1]));
    let replica = Replica::open(&following.file).unwrap();
    assert_eq!(replica.message_count(), 12);
}

#[tokio::test]
async fn an_edit_while_followed_replaces_the_message() {
    let board = FakeBoard::with_history(history_across_two_channels());
    let mut following = copy_then_follow(&board).await.unwrap();

    board.change_history(|history| {
        history.edit("12", "looking, it was the cache");
        history.post("17", "fleet/www", "13", 7, "green again");
    });

    assert!(until_reported(&mut following.states, &ReplicaState::Live { stored: 8 }).await);
    following.task.abort();
    let replica = Replica::open(&following.file).unwrap();
    assert_eq!(
        replica.stored_view("12").unwrap().unwrap()["text"],
        "looking, it was the cache"
    );
}

#[tokio::test]
async fn the_feed_cursor_is_kept_as_the_feed_advances_even_past_bookkeeping() {
    let board = FakeBoard::with_history(history_across_two_channels());
    let mut following = copy_then_follow(&board).await.unwrap();

    board.change_history(|history| {
        history.bookkeeping();
        history.post("17", "fleet/www", "13", 7, "green again");
    });

    assert!(until_reported(&mut following.states, &ReplicaState::Live { stored: 8 }).await);
    following.task.abort();
    let replica = Replica::open(&following.file).unwrap();
    assert_eq!(
        replica.feed_cursor().unwrap(),
        Some(String::from("event-8"))
    );
}

#[tokio::test]
async fn a_refused_stream_is_reported_and_retried() {
    let board = FakeBoard::with_history(history_across_two_channels());
    board
        .streams_to_refuse
        .store(2, std::sync::atomic::Ordering::SeqCst);
    let mut following = copy_then_follow(&board).await.unwrap();

    assert!(
        until_reported(
            &mut following.states,
            &ReplicaState::Reconnecting { stored: 7 }
        )
        .await
    );
    board.change_history(|history| history.post("17", "fleet/www", "13", 7, "green again"));

    assert!(until_reported(&mut following.states, &ReplicaState::Live { stored: 8 }).await);
    following.task.abort();
}

#[tokio::test]
async fn a_board_that_ends_every_stream_at_once_is_asked_again_ever_more_slowly() {
    let board = FakeBoard::with_history(history_across_two_channels());
    board
        .streams_end_at_once
        .store(true, std::sync::atomic::Ordering::SeqCst);
    let pacing = Pacing::new(Duration::from_millis(10), Duration::from_millis(40));
    let mut following = copy_then_follow_at(&board, pacing).await.unwrap();

    assert!(
        until_reported(
            &mut following.states,
            &ReplicaState::Reconnecting { stored: 7 }
        )
        .await
    );
    tokio::time::sleep(Duration::from_millis(200)).await;
    following.task.abort();

    let streams_opened = board.stream_starts.lock().unwrap().len();
    assert!(streams_opened <= 12, "{streams_opened} streams opened");
}

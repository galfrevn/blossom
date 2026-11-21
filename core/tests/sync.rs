#[path = "support/answers.rs"]
mod answers;
#[path = "support/history.rs"]
mod history;

use std::convert::Infallible;
use std::error::Error;
use std::path::PathBuf;
use std::time::Duration;

use blossom::contract::{ReplicaState, StopReason};
use blossom::replica::Replica;
use blossom::{
    BoardAccess, CopiedBoard, FollowError, KeepingEnded, Pacing, copy_board, follow_feed,
    keep_in_step,
};
use history::{BoardHistory, FakeBoard, serve_board};
use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
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

fn quick_pacing() -> Pacing {
    Pacing {
        first_pause: Duration::from_millis(1),
        longest_pause: Duration::from_millis(5),
        failures_before_asking_the_board: 3,
        longest_wait_for_a_stream: Duration::from_millis(200),
        longest_silence: Duration::from_secs(5),
    }
}

async fn copy_of(history: &BoardHistory) -> Result<Copy, Box<dyn Error>> {
    copy_from(&FakeBoard::with_history(history.clone())).await
}

async fn copy_from(board: &FakeBoard) -> Result<Copy, Box<dyn Error>> {
    let server = MockServer::start().await;
    serve_board(&server, board.clone()).await;
    let access = BoardAccess::new(format!("{}/api/board", server.uri()), "token-of-ada");
    let folder = TempDir::new()?;
    let replica = Replica::open(&folder.path().join("board.sqlite"))?;
    let mut reported_states = Vec::new();

    let copied = copy_board(&access, replica, quick_pacing(), |state| {
        reported_states.push(state);
    })
    .await?;

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

#[tokio::test]
async fn pages_the_board_refuses_are_asked_for_again_while_reporting_reconnecting() {
    let board = FakeBoard::with_history(history_across_two_channels());
    board
        .pages_to_refuse
        .store(4, std::sync::atomic::Ordering::SeqCst);

    let copy = copy_from(&board).await.unwrap();

    assert_eq!(copy.copied.replica.message_count(), 7);
    assert!(
        copy.reported_states
            .contains(&ReplicaState::Reconnecting { stored: 0 })
    );
}

#[tokio::test]
async fn a_page_slower_than_a_quick_answer_is_waited_for() {
    let board = FakeBoard::with_history(history_across_two_channels());
    board
        .pages_to_slow_down
        .store(1, std::sync::atomic::Ordering::SeqCst);

    let copy = copy_from(&board).await.unwrap();

    assert_eq!(copy.copied.replica.message_count(), 7);
    let reconnected = copy
        .reported_states
        .iter()
        .any(|state| matches!(state, ReplicaState::Reconnecting { .. }));
    assert!(!reconnected);
}

struct Following {
    _server: MockServer,
    _folder: TempDir,
    file: PathBuf,
    states: UnboundedReceiver<ReplicaState>,
    task: JoinHandle<Result<Infallible, FollowError>>,
}

async fn copy_then_follow(board: &FakeBoard) -> Result<Following, Box<dyn Error>> {
    copy_then_follow_at(board, quick_pacing()).await
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
    let copied = copy_board(&access, replica, quick_pacing(), |_| {}).await?;
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
    let pacing = Pacing {
        first_pause: Duration::from_millis(10),
        longest_pause: Duration::from_millis(40),
        ..quick_pacing()
    };
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

struct Keeping {
    server: Option<MockServer>,
    _folder: TempDir,
    file: PathBuf,
    states: UnboundedReceiver<ReplicaState>,
    task: JoinHandle<KeepingEnded>,
}

async fn start_keeping_in_step(board: &FakeBoard) -> Result<Keeping, Box<dyn Error>> {
    let server = MockServer::builder().start().await;
    serve_board(&server, board.clone()).await;
    let access = BoardAccess::new(format!("{}/api/board", server.uri()), "token-of-ada");
    let folder = TempDir::new()?;
    let file = folder.path().join("board.sqlite");
    let (sender, states) = unbounded_channel();
    let file_to_keep = file.clone();
    let task = tokio::spawn(async move {
        keep_in_step(&access, &file_to_keep, quick_pacing(), |state| {
            sender.send(state).ok();
        })
        .await
    });
    Ok(Keeping {
        server: Some(server),
        _folder: folder,
        file,
        states,
        task,
    })
}

async fn until_ended(task: JoinHandle<KeepingEnded>) -> Option<KeepingEnded> {
    tokio::time::timeout(Duration::from_secs(5), task)
        .await
        .ok()?
        .ok()
}

#[tokio::test]
async fn a_board_that_stops_answering_while_followed_is_gone_and_its_replica_kept() {
    let board = FakeBoard::with_history(history_across_two_channels());
    let mut keeping = start_keeping_in_step(&board).await.unwrap();
    assert!(until_reported(&mut keeping.states, &ReplicaState::Live { stored: 7 }).await);

    drop(keeping.server.take());

    let ended = until_ended(keeping.task).await;
    assert!(matches!(ended, Some(KeepingEnded::BoardGone)));
    assert_eq!(Replica::open(&keeping.file).unwrap().message_count(), 7);
}

#[tokio::test]
async fn a_board_that_stops_answering_while_copied_is_gone() {
    let board = FakeBoard::with_history(history_across_two_channels());
    board
        .pages_to_refuse
        .store(usize::MAX, std::sync::atomic::Ordering::SeqCst);
    let mut keeping = start_keeping_in_step(&board).await.unwrap();
    assert!(
        until_reported(
            &mut keeping.states,
            &ReplicaState::Reconnecting { stored: 0 }
        )
        .await
    );

    drop(keeping.server.take());

    let ended = until_ended(keeping.task).await;
    assert!(matches!(ended, Some(KeepingEnded::BoardGone)));
}

#[tokio::test]
async fn a_board_that_refuses_its_feed_but_still_answers_is_not_gone() {
    let board = FakeBoard::with_history(history_across_two_channels());
    board
        .streams_to_refuse
        .store(usize::MAX, std::sync::atomic::Ordering::SeqCst);
    let mut keeping = start_keeping_in_step(&board).await.unwrap();
    assert!(
        until_reported(
            &mut keeping.states,
            &ReplicaState::Reconnecting { stored: 7 }
        )
        .await
    );

    tokio::time::sleep(Duration::from_millis(100)).await;

    assert!(!keeping.task.is_finished());
    keeping.task.abort();
}

#[tokio::test]
async fn an_event_the_core_cannot_read_stops_the_replica_and_says_so() {
    let board = FakeBoard::with_history(history_across_two_channels());
    let mut keeping = start_keeping_in_step(&board).await.unwrap();
    assert!(until_reported(&mut keeping.states, &ReplicaState::Live { stored: 7 }).await);

    board.change_history(BoardHistory::unreadable_event);

    let stopped = ReplicaState::Stopped {
        stored: 7,
        reason: StopReason::UnreadableBoard,
    };
    assert!(until_reported(&mut keeping.states, &stopped).await);
    let ended = until_ended(keeping.task).await;
    assert!(matches!(ended, Some(KeepingEnded::Stopped(_))));
}

#[tokio::test]
async fn a_replica_that_cannot_be_opened_stops_and_says_so() {
    let folder = TempDir::new().unwrap();
    std::fs::write(folder.path().join("blocker"), "").unwrap();
    let file = folder.path().join("blocker").join("board.sqlite");
    let access = BoardAccess::new("http://127.0.0.1:9/api/board", "token-of-ada");
    let mut reported_states = Vec::new();

    let ended = keep_in_step(&access, &file, quick_pacing(), |state| {
        reported_states.push(state);
    })
    .await;

    assert!(matches!(ended, KeepingEnded::Stopped(_)));
    assert_eq!(
        reported_states,
        vec![ReplicaState::Stopped {
            stored: 0,
            reason: StopReason::UnwritableReplica,
        }]
    );
}

#[tokio::test]
async fn a_stream_the_board_does_not_open_in_time_counts_as_a_refusal() {
    let board = FakeBoard::with_history(history_across_two_channels());
    board
        .streams_to_stall
        .store(1, std::sync::atomic::Ordering::SeqCst);
    let mut following = copy_then_follow(&board).await.unwrap();

    assert!(
        until_reported(
            &mut following.states,
            &ReplicaState::Reconnecting { stored: 7 }
        )
        .await
    );
    assert!(until_reported(&mut following.states, &ReplicaState::Live { stored: 7 }).await);
    following.task.abort();
}

async fn start_board_that_falls_silent() -> Result<String, Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let url = format!("http://{}/api/board", listener.local_addr()?);
    tokio::spawn(async move {
        while let Ok((mut connection, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut request = vec![0; 4096];
                let request_size = connection.read(&mut request).await.unwrap_or(0);
                if request_size == 0 {
                    return;
                }
                let opening =
                    "HTTP/1.1 200 OK\r\ncontent-type: text/event-stream\r\n\r\n: connected\n\n";
                connection.write_all(opening.as_bytes()).await.ok();
                tokio::time::sleep(Duration::from_secs(60)).await;
            });
        }
    });
    Ok(url)
}

#[tokio::test]
async fn a_stream_that_falls_silent_counts_as_a_failure() {
    let url = start_board_that_falls_silent().await.unwrap();
    let access = BoardAccess::new(url, "token-of-ada");
    let folder = TempDir::new().unwrap();
    let replica = Replica::open(&folder.path().join("board.sqlite")).unwrap();
    let pacing = Pacing {
        longest_silence: Duration::from_millis(50),
        ..quick_pacing()
    };
    let (sender, mut states) = unbounded_channel();
    let task = tokio::spawn(async move {
        follow_feed(&access, replica, String::from("event-0"), pacing, |state| {
            sender.send(state).ok();
        })
        .await
    });

    assert!(until_reported(&mut states, &ReplicaState::Live { stored: 0 }).await);
    assert!(until_reported(&mut states, &ReplicaState::Reconnecting { stored: 0 }).await);
    task.abort();
}

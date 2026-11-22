#[path = "support/answers.rs"]
mod answers;
#[path = "support/history.rs"]
mod history;
#[path = "support/outdated.rs"]
mod outdated;

use std::convert::Infallible;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::time::Duration;

use blossom::contract::{ReplicaState, StopReason};
use blossom::reader::ReplicaReader;
use blossom::replica::{Actor, Batch, Claim, Replica, Via, new_replica_file};
use blossom::{
    BoardAccess, CopiedBoard, FollowError, KeepingEnded, Pacing, copy_board, follow_feed,
    keep_in_step,
};
use history::{
    BoardHistory, FakeActor, FakeBoard, asked_actor_ids, lease_until, serve_board, time_at,
};
use outdated::write_a_replica_from_before_task_events;
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
    server: MockServer,
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

    let copied = copy_board(
        &access,
        replica,
        quick_pacing(),
        |state| {
            reported_states.push(state);
        },
        || {},
    )
    .await?;

    Ok(Copy {
        server,
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

fn ids_in_flow(file: &Path) -> Option<Vec<String>> {
    let reader = ReplicaReader::open(file).ok()?;
    let window = reader.flow_window_in(None, 0, 100).ok()?;
    Some(window.rows.into_iter().map(|row| row.id).collect())
}

fn is_whole(file: &Path) -> Option<bool> {
    let replica = Replica::open(file).ok()?;
    let feed_cursor = replica.feed_cursor().ok()?;
    Some(feed_cursor.is_some())
}

fn the_two_newest_threads_with_their_replies() -> Vec<String> {
    ["10", "12", "13", "15", "16"].map(String::from).to_vec()
}

#[tokio::test]
async fn the_newest_threads_are_stored_with_their_replies_before_the_feed_is_replayed() {
    let server = MockServer::start().await;
    serve_board(
        &server,
        FakeBoard::with_history(history_across_two_channels()),
    )
    .await;
    let access = BoardAccess::new(format!("{}/api/board", server.uri()), "token-of-ada");
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let replica = Replica::open(&file).unwrap();
    let mut stored_first = None;

    let copied = copy_board(
        &access,
        replica,
        quick_pacing(),
        |_| {},
        || {
            stored_first = ids_in_flow(&file);
        },
    )
    .await
    .unwrap();

    assert_eq!(
        stored_first,
        Some(the_two_newest_threads_with_their_replies())
    );
    assert_eq!(copied.replica.message_count(), 7);
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
    history.hidden_event();
    history.hidden_event();
    history.hidden_event();
    history.hidden_event();
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
    server: MockServer,
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
    let copied = copy_board(&access, replica, quick_pacing(), |_| {}, || {}).await?;
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
        server,
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
    until_seen(states, |state| state == expected).await
}

async fn until_seen(
    states: &mut UnboundedReceiver<ReplicaState>,
    wanted: impl Fn(&ReplicaState) -> bool,
) -> bool {
    let waiting = async {
        while let Some(state) = states.recv().await {
            if wanted(&state) {
                return true;
            }
        }
        false
    };
    tokio::time::timeout(Duration::from_secs(5), waiting)
        .await
        .unwrap_or(false)
}

fn copying(state: &ReplicaState) -> bool {
    matches!(state, ReplicaState::Copying { .. })
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
async fn the_feed_cursor_is_kept_as_the_feed_advances_even_past_events_it_does_not_show() {
    let board = FakeBoard::with_history(history_across_two_channels());
    let mut following = copy_then_follow(&board).await.unwrap();

    board.change_history(|history| {
        history.hidden_event();
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
    folder: TempDir,
    states: UnboundedReceiver<ReplicaState>,
    task: JoinHandle<KeepingEnded>,
}

async fn start_keeping_in_step(board: &FakeBoard) -> Result<Keeping, Box<dyn Error>> {
    let server = MockServer::builder().start().await;
    serve_board(&server, board.clone()).await;
    let folder = TempDir::new()?;
    let (states, task) = keep_in_step_in(&server, folder.path());
    Ok(Keeping {
        server: Some(server),
        folder,
        states,
        task,
    })
}

fn keep_in_step_in(
    server: &MockServer,
    replicas_folder: &Path,
) -> (UnboundedReceiver<ReplicaState>, JoinHandle<KeepingEnded>) {
    let access = BoardAccess::new(format!("{}/api/board", server.uri()), "token-of-ada");
    let replicas_folder = replicas_folder.to_path_buf();
    let (sender, states) = unbounded_channel();
    let task = tokio::spawn(async move {
        keep_in_step(
            &access,
            &replicas_folder,
            quick_pacing(),
            |_| {},
            |state| {
                sender.send(state).ok();
            },
        )
        .await
    });
    (states, task)
}

fn replica_files(replicas_folder: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(replicas_folder)? {
        let file = entry?.path();
        if file
            .extension()
            .is_some_and(|extension| extension == "sqlite")
        {
            files.push(file);
        }
    }
    files.sort();
    Ok(files)
}

async fn threads_read(server: &MockServer) -> usize {
    let requests = server.received_requests().await.unwrap_or_default();
    requests
        .iter()
        .filter(|request| request.url.path() == "/api/board/threads")
        .count()
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
    let files = replica_files(keeping.folder.path()).unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(Replica::open(&files[0]).unwrap().message_count(), 7);
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
    let replicas_folder = folder.path().join("blocker").join("replicas");
    let access = BoardAccess::new("http://127.0.0.1:9/api/board", "token-of-ada");
    let mut reported_states = Vec::new();

    let ended = keep_in_step(
        &access,
        &replicas_folder,
        quick_pacing(),
        |_| {},
        |state| {
            reported_states.push(state);
        },
    )
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

fn another_project_history() -> BoardHistory {
    let mut history = BoardHistory::default();
    history.post("10", "fleet/docs", "10", 40, "draft the guide");
    history.post("11", "fleet/docs", "10", 41, "reviewing");
    history.post("12", "fleet/docs", "12", 42, "publish");
    history
}

#[tokio::test]
async fn reopening_a_known_board_follows_on_without_copying_it_again() {
    let board = FakeBoard::with_history(history_across_two_channels());
    let mut first = start_keeping_in_step(&board).await.unwrap();
    assert!(until_reported(&mut first.states, &ReplicaState::Live { stored: 7 }).await);
    first.task.abort();
    let server = first.server.take().unwrap();
    let threads_read_by_the_copy = threads_read(&server).await;
    board.change_history(|history| history.post("17", "fleet/www", "13", 7, "green again"));

    let (mut states, task) = keep_in_step_in(&server, first.folder.path());

    assert!(until_reported(&mut states, &ReplicaState::Live { stored: 8 }).await);
    task.abort();
    assert_eq!(threads_read(&server).await, threads_read_by_the_copy);
    assert_eq!(replica_files(first.folder.path()).unwrap().len(), 1);
}

#[tokio::test]
async fn a_refused_cursor_copies_the_board_again() {
    let board = FakeBoard::with_history(history_across_two_channels());
    let mut first = start_keeping_in_step(&board).await.unwrap();
    assert!(until_reported(&mut first.states, &ReplicaState::Live { stored: 7 }).await);
    first.task.abort();
    let server = first.server.take().unwrap();
    let file = replica_files(first.folder.path()).unwrap().remove(0);
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply_from_feed(&Batch::default(), "a-cursor-the-board-never-gave")
        .unwrap();
    drop(replica);

    let (mut states, task) = keep_in_step_in(&server, first.folder.path());

    assert!(until_seen(&mut states, copying).await);
    assert!(until_reported(&mut states, &ReplicaState::Live { stored: 7 }).await);
    task.abort();
    let files = replica_files(first.folder.path()).unwrap();
    assert_eq!(files.len(), 1);
    let cursor = Replica::open(&files[0]).unwrap().feed_cursor().unwrap();
    assert_ne!(cursor.as_deref(), Some("a-cursor-the-board-never-gave"));
}

#[tokio::test]
async fn a_different_board_at_the_same_address_gets_a_replica_of_its_own() {
    let board = FakeBoard::with_history(history_across_two_channels());
    let mut first = start_keeping_in_step(&board).await.unwrap();
    assert!(until_reported(&mut first.states, &ReplicaState::Live { stored: 7 }).await);
    first.task.abort();
    let server = first.server.take().unwrap();
    let first_replica = replica_files(first.folder.path()).unwrap().remove(0);
    board.change_history(|history| *history = another_project_history());

    let (mut states, task) = keep_in_step_in(&server, first.folder.path());

    assert!(until_seen(&mut states, copying).await);
    assert!(until_reported(&mut states, &ReplicaState::Live { stored: 3 }).await);
    task.abort();
    assert_eq!(replica_files(first.folder.path()).unwrap().len(), 2);
    assert_eq!(Replica::open(&first_replica).unwrap().message_count(), 7);
}

#[tokio::test]
async fn an_empty_board_is_copied_again_when_reopened() {
    let board = FakeBoard::with_history(BoardHistory::default());
    let mut first = start_keeping_in_step(&board).await.unwrap();
    assert!(until_reported(&mut first.states, &ReplicaState::Live { stored: 0 }).await);
    first.task.abort();
    let server = first.server.take().unwrap();

    let (mut states, task) = keep_in_step_in(&server, first.folder.path());

    assert!(until_seen(&mut states, copying).await);
    task.abort();
    assert_eq!(replica_files(first.folder.path()).unwrap().len(), 1);
}

#[tokio::test]
async fn a_cursor_refused_right_after_a_copy_stops_instead_of_copying_in_a_loop() {
    let board = FakeBoard::with_history(history_across_two_channels());
    board
        .streams_refuse_every_cursor
        .store(true, std::sync::atomic::Ordering::SeqCst);
    let mut keeping = start_keeping_in_step(&board).await.unwrap();

    let stopped = ReplicaState::Stopped {
        stored: 7,
        reason: StopReason::UnreadableBoard,
    };
    assert!(until_reported(&mut keeping.states, &stopped).await);
    let ended = until_ended(keeping.task).await;
    assert!(matches!(ended, Some(KeepingEnded::Stopped(_))));
}

struct Announcement {
    file: PathBuf,
    shown: Option<Vec<String>>,
    whole: Option<bool>,
}

#[tokio::test]
async fn the_replica_being_copied_is_announced_once_it_holds_the_newest_threads() {
    let board = FakeBoard::with_history(history_across_two_channels());
    let server = MockServer::builder().start().await;
    serve_board(&server, board.clone()).await;
    let folder = TempDir::new().unwrap();
    let access = BoardAccess::new(format!("{}/api/board", server.uri()), "token-of-ada");
    let replicas_folder = folder.path().to_path_buf();
    let (announcements, mut announced) = unbounded_channel();
    let (reports, mut states) = unbounded_channel();
    let task = tokio::spawn(async move {
        keep_in_step(
            &access,
            &replicas_folder,
            quick_pacing(),
            |file| {
                announcements
                    .send(Announcement {
                        file: file.to_path_buf(),
                        shown: ids_in_flow(file),
                        whole: is_whole(file),
                    })
                    .ok();
            },
            |state| {
                reports.send(state).ok();
            },
        )
        .await
    });

    assert!(until_reported(&mut states, &ReplicaState::Live { stored: 7 }).await);
    task.abort();

    let announcement = announced.recv().await.unwrap();
    assert_eq!(
        announcement.shown,
        Some(the_two_newest_threads_with_their_replies())
    );
    assert_eq!(announcement.whole, Some(false));
    assert!(announced.try_recv().is_err());
    assert_eq!(
        replica_files(folder.path()).unwrap(),
        vec![announcement.file]
    );
}

#[tokio::test]
async fn a_copy_stopped_halfway_is_copied_again_and_never_followed_as_a_whole_one() {
    let board = FakeBoard::with_history(history_across_two_channels());
    let server = MockServer::builder().start().await;
    serve_board(&server, board.clone()).await;
    let folder = TempDir::new().unwrap();
    let access = BoardAccess::new(format!("{}/api/board", server.uri()), "token-of-ada");
    let replicas_folder = folder.path().to_path_buf();
    let board_that_goes_quiet = board.clone();
    let (halfway, mut reached_halfway) = unbounded_channel();
    let quitting = tokio::spawn(async move {
        keep_in_step(
            &access,
            &replicas_folder,
            quick_pacing(),
            |_| {
                board_that_goes_quiet
                    .pages_to_refuse
                    .store(usize::MAX, std::sync::atomic::Ordering::SeqCst);
                halfway.send(()).ok();
            },
            |_| {},
        )
        .await
    });
    let stopped = tokio::time::timeout(Duration::from_secs(5), reached_halfway.recv()).await;
    assert_eq!(stopped, Ok(Some(())));
    quitting.abort();
    assert!(quitting.await.is_err());
    let half_copies = replica_files(folder.path()).unwrap();
    assert_eq!(half_copies.len(), 1);
    assert_eq!(is_whole(&half_copies[0]), Some(false));
    board
        .pages_to_refuse
        .store(0, std::sync::atomic::Ordering::SeqCst);

    let (mut states, task) = keep_in_step_in(&server, folder.path());

    assert!(until_seen(&mut states, copying).await);
    assert!(until_reported(&mut states, &ReplicaState::Live { stored: 7 }).await);
    task.abort();
    let files = replica_files(folder.path()).unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(is_whole(&files[0]), Some(true));
}

fn julian() -> FakeActor {
    FakeActor::person("julian", "Julian Benegas")
}

fn triage() -> FakeActor {
    FakeActor::agent("triage", "julian")
}

fn claude() -> FakeActor {
    FakeActor::agent("claude", "julian")
}

fn codex() -> FakeActor {
    FakeActor::agent("codex", "julian")
}

const DEPLOYER_NO_ONE_LISTS: &str = "local:user:julian+deployer";

fn history_of_every_task_change() -> BoardHistory {
    let mut history = BoardHistory::default();
    history.know_actor(julian());
    history.post("10", "work", "10", 0, "fix CVE-123 in acme/web");
    history.post("11", "work", "11", 1, "review the fix for CVE-123");
    history.post("12", "work", "12", 2, "rotate the deploy keys");
    history.post("13", "work", "13", 3, "deploy acme/web to production");
    history.create_task("10", &triage(), 4, None, &[]);
    history.claim_task("10", &claude(), 5);
    history.block_task("10", &claude(), 6, "needs_input", "which version?");
    history.unblock_task("10", &triage(), 7);
    history.claim_task("10", &claude(), 8);
    history.fail_task("10", &claude(), 9, "the tests fail");
    history.reopen_task("10", &triage(), 10);
    history.claim_task("10", &codex(), 11);
    history.complete_task("10", &codex(), 12, "fixed in a1b2c3, tests pass");
    history.create_task("11", &triage(), 13, None, &[]);
    history.claim_task("11", &claude(), 14);
    history.expire_task("11", 20);
    history.claim_task("11", &claude(), 21);
    history.create_task("12", &triage(), 22, None, &[]);
    history.claim_task("12", &codex(), 23);
    history.release_task("12", &codex(), 24);
    history.cancel_task("12", &triage(), 25, "superseded by the nightly rotation");
    history.create_task("13", &triage(), 26, Some(DEPLOYER_NO_ONE_LISTS), &["11"]);
    history
}

#[derive(Debug, PartialEq)]
struct TaskChange {
    event_type: String,
    actor: Option<String>,
    created_at: String,
}

fn changes_of(replica: &Replica, task: &str) -> Result<Vec<TaskChange>, Box<dyn Error>> {
    let changes = replica
        .task_events(task)?
        .into_iter()
        .map(|event| TaskChange {
            event_type: event.event_type,
            actor: event.actor,
            created_at: event.created_at,
        })
        .collect();
    Ok(changes)
}

fn change(event_type: &str, actor: Option<&FakeActor>, minute: u32) -> TaskChange {
    TaskChange {
        event_type: String::from(event_type),
        actor: actor.map(|actor| actor.id.clone()),
        created_at: time_at(minute),
    }
}

async fn actor_ids_asked_for(server: &MockServer) -> Vec<String> {
    let requests = server.received_requests().await.unwrap_or_default();
    requests
        .iter()
        .filter(|request| request.url.path() == "/api/board/actors")
        .flat_map(asked_actor_ids)
        .collect()
}

#[tokio::test]
async fn every_task_change_is_copied_as_a_task_event_with_its_actor_and_time() {
    let copy = copy_of(&history_of_every_task_change()).await.unwrap();
    let replica = &copy.copied.replica;

    assert_eq!(
        changes_of(replica, "10").unwrap(),
        vec![
            change("task.created", Some(&triage()), 4),
            change("task.claimed", Some(&claude()), 5),
            change("task.blocked", Some(&claude()), 6),
            change("task.unblocked", Some(&triage()), 7),
            change("task.claimed", Some(&claude()), 8),
            change("task.failed", Some(&claude()), 9),
            change("task.reopened", Some(&triage()), 10),
            change("task.claimed", Some(&codex()), 11),
            change("task.completed", Some(&codex()), 12),
        ]
    );
    assert_eq!(
        changes_of(replica, "11").unwrap(),
        vec![
            change("task.created", Some(&triage()), 13),
            change("task.claimed", Some(&claude()), 14),
            change("task.expired", None, 20),
            change("task.claimed", Some(&claude()), 21),
        ]
    );
    assert_eq!(
        changes_of(replica, "12").unwrap(),
        vec![
            change("task.created", Some(&triage()), 22),
            change("task.claimed", Some(&codex()), 23),
            change("task.released", Some(&codex()), 24),
            change("task.canceled", Some(&triage()), 25),
        ]
    );
}

#[tokio::test]
async fn the_fields_of_each_task_change_are_copied() {
    let copy = copy_of(&history_of_every_task_change()).await.unwrap();
    let replica = &copy.copied.replica;

    let fixing = replica.task_events("10").unwrap();
    let rotating = replica.task_events("12").unwrap();
    let deploying = replica.task_events("13").unwrap();
    assert_eq!(fixing[2].reason.as_deref(), Some("which version?"));
    assert_eq!(fixing[2].payload["kind"], "needs_input");
    assert_eq!(fixing[5].reason.as_deref(), Some("the tests fail"));
    assert_eq!(
        fixing[8].result.as_deref(),
        Some("fixed in a1b2c3, tests pass")
    );
    assert_eq!(
        rotating[3].reason.as_deref(),
        Some("superseded by the nightly rotation")
    );
    assert_eq!(
        deploying[0].assigned_to.as_deref(),
        Some(DEPLOYER_NO_ONE_LISTS)
    );
    assert_eq!(deploying[0].needs, Some(vec![String::from("11")]));
}

#[tokio::test]
async fn the_task_index_is_copied_as_each_task_stands_now() {
    let copy = copy_of(&history_of_every_task_change()).await.unwrap();
    let replica = &copy.copied.replica;

    let fixing = replica.task("10").unwrap().unwrap();
    let reviewing = replica.task("11").unwrap().unwrap();
    let rotating = replica.task("12").unwrap().unwrap();
    let deploying = replica.task("13").unwrap().unwrap();
    assert_eq!(fixing.state, "completed");
    assert_eq!(fixing.channel, "work");
    assert_eq!(fixing.owner, triage().principal());
    assert_eq!(fixing.made_as, triage().id);
    assert_eq!(
        fixing.result.as_deref(),
        Some("fixed in a1b2c3, tests pass")
    );
    assert_eq!(fixing.claim, None);
    assert_eq!(fixing.latest_event_at, Some(time_at(12)));
    assert_eq!(reviewing.state, "claimed");
    assert_eq!(
        reviewing.claim,
        Some(Claim {
            actor: claude().id,
            until: lease_until(21),
        })
    );
    assert_eq!(rotating.state, "canceled");
    assert_eq!(
        rotating.reason.as_deref(),
        Some("superseded by the nightly rotation")
    );
    assert_eq!(deploying.state, "waiting");
    assert_eq!(
        deploying.assigned_to.as_deref(),
        Some(DEPLOYER_NO_ONE_LISTS)
    );
    assert_eq!(deploying.needs, vec![String::from("11")]);
    assert_eq!(deploying.latest_event_at, Some(time_at(26)));
}

#[tokio::test]
async fn the_actors_behind_task_changes_and_messages_are_copied() {
    let copy = copy_of(&history_of_every_task_change()).await.unwrap();
    let replica = &copy.copied.replica;

    assert_eq!(
        replica.actor(&claude().id).unwrap(),
        Some(Actor {
            id: claude().id,
            kind: String::from("agent"),
            handle: String::from("julian/claude"),
            name: String::from("claude"),
            via: Some(Via {
                name: String::from("julian"),
                environment: None,
            }),
        })
    );
    assert_eq!(
        replica.actor("local:user:ada").unwrap().unwrap().name,
        "Ada Lovelace"
    );
}

#[tokio::test]
async fn an_owner_never_seen_in_an_event_is_asked_for_by_its_id() {
    let copy = copy_of(&history_of_every_task_change()).await.unwrap();

    assert!(
        actor_ids_asked_for(&copy.server)
            .await
            .contains(&julian().id)
    );
    assert_eq!(
        copy.copied.replica.actor(&julian().id).unwrap(),
        Some(Actor {
            id: julian().id,
            kind: String::from("person"),
            handle: String::from("julian"),
            name: String::from("Julian Benegas"),
            via: None,
        })
    );
}

#[tokio::test]
async fn an_actor_the_board_does_not_list_is_asked_for_once() {
    let board = FakeBoard::with_history(history_of_every_task_change());
    let mut following = copy_then_follow(&board).await.unwrap();

    board.change_history(|history| {
        history.block_task("13", &triage(), 30, "awaiting_approval", "deploy now?");
        history.unblock_task("13", &triage(), 31);
        history.post("14", "work", "13", 32, "approved");
    });

    assert!(until_reported(&mut following.states, &ReplicaState::Live { stored: 5 }).await);
    following.task.abort();
    let asked_for_the_deployer = actor_ids_asked_for(&following.server)
        .await
        .into_iter()
        .filter(|id| id == DEPLOYER_NO_ONE_LISTS)
        .count();
    assert_eq!(asked_for_the_deployer, 1);
    let replica = Replica::open(&following.file).unwrap();
    assert_eq!(replica.actor(DEPLOYER_NO_ONE_LISTS).unwrap(), None);
    assert_eq!(replica.task_events("13").unwrap().len(), 3);
}

#[tokio::test]
async fn task_changes_while_followed_reach_the_replica() {
    let board = FakeBoard::with_history(history_of_every_task_change());
    let mut following = copy_then_follow(&board).await.unwrap();

    board.change_history(|history| {
        history.complete_task("11", &claude(), 30, "reviewed, looks good");
        history.post("14", "work", "11", 31, "merged");
    });

    assert!(until_reported(&mut following.states, &ReplicaState::Live { stored: 5 }).await);
    following.task.abort();
    let replica = Replica::open(&following.file).unwrap();
    let reviewing = replica.task("11").unwrap().unwrap();
    assert_eq!(reviewing.state, "completed");
    assert_eq!(reviewing.claim, None);
    assert_eq!(reviewing.result.as_deref(), Some("reviewed, looks good"));
    assert_eq!(reviewing.latest_event_at, Some(time_at(30)));
    assert_eq!(
        changes_of(&replica, "11").unwrap().last(),
        Some(&change("task.completed", Some(&claude()), 30))
    );
}

fn grace() -> FakeActor {
    FakeActor::person("grace", "Grace Hopper")
}

fn builder() -> FakeActor {
    FakeActor::agent("builder", "grace")
}

fn a_task_made_by_a_new_agent(history: &mut BoardHistory) {
    history.know_actor(grace());
    history.post("14", "docs", "14", 30, "build the docs");
    history.create_task("14", &builder(), 31, None, &[]);
    history.post("15", "docs", "14", 32, "building");
}

#[tokio::test]
async fn actors_first_seen_while_followed_are_kept_and_unseen_owners_asked_for() {
    let board = FakeBoard::with_history(history_of_every_task_change());
    let mut following = copy_then_follow(&board).await.unwrap();

    board.change_history(a_task_made_by_a_new_agent);

    assert!(until_reported(&mut following.states, &ReplicaState::Live { stored: 6 }).await);
    following.task.abort();
    let replica = Replica::open(&following.file).unwrap();
    assert_eq!(
        replica.actor(&builder().id).unwrap().unwrap().via,
        Some(Via {
            name: String::from("grace"),
            environment: None,
        })
    );
    assert_eq!(
        replica.actor(&grace().id).unwrap().unwrap().name,
        "Grace Hopper"
    );
}

#[tokio::test]
async fn actors_the_board_fails_to_tell_while_followed_are_asked_for_again() {
    let board = FakeBoard::with_history(history_of_every_task_change());
    let mut following = copy_then_follow(&board).await.unwrap();
    board
        .pages_to_refuse
        .store(1, std::sync::atomic::Ordering::SeqCst);

    board.change_history(a_task_made_by_a_new_agent);

    assert!(until_reported(&mut following.states, &ReplicaState::Live { stored: 6 }).await);
    following.task.abort();
    let replica = Replica::open(&following.file).unwrap();
    assert_eq!(
        replica.actor(&grace().id).unwrap().unwrap().name,
        "Grace Hopper"
    );
    assert_eq!(replica.task_events("14").unwrap().len(), 1);
    assert_eq!(replica.task("14").unwrap().unwrap().owner, grace().id);
}

#[tokio::test]
async fn a_replica_made_before_task_events_were_kept_is_copied_again() {
    let board = FakeBoard::with_history(history_of_every_task_change());
    let server = MockServer::builder().start().await;
    serve_board(&server, board.clone()).await;
    let folder = TempDir::new().unwrap();
    let access = BoardAccess::new(format!("{}/api/board", server.uri()), "token-of-ada");
    let outdated_file = new_replica_file(folder.path(), &access).unwrap();
    write_a_replica_from_before_task_events(&outdated_file, "10", &time_at(0), "event-25").unwrap();

    let (mut states, task) = keep_in_step_in(&server, folder.path());

    assert!(until_seen(&mut states, copying).await);
    assert!(until_reported(&mut states, &ReplicaState::Live { stored: 4 }).await);
    task.abort();
    let files = replica_files(folder.path()).unwrap();
    assert_eq!(files.len(), 1);
    assert!(!outdated_file.exists());
    let replica = Replica::open(&files[0]).unwrap();
    assert_eq!(replica.task("10").unwrap().unwrap().state, "completed");
}

fn a_task_claimed_by_claude() -> BoardHistory {
    let mut history = BoardHistory::default();
    history.post("10", "work", "10", 0, "deploy www");
    history.create_task("10", &julian(), 1, None, &[]);
    history.claim_task("10", &claude(), 2);
    history
}

#[tokio::test]
async fn the_copy_keeps_a_renewal_among_its_task_s_events() {
    let mut history = a_task_claimed_by_claude();
    history.renew_task("10", &claude(), 5);

    let copy = copy_of(&history).await.unwrap();

    let kept: Vec<String> = copy
        .copied
        .replica
        .task_events("10")
        .unwrap()
        .into_iter()
        .map(|event| event.event_type)
        .collect();
    assert_eq!(kept, ["task.created", "task.claimed", "task.renewed"]);
}

#[tokio::test]
async fn a_renewal_in_the_feed_moves_the_lease_of_its_claim() {
    let board = FakeBoard::with_history(a_task_claimed_by_claude());
    let mut following = copy_then_follow(&board).await.unwrap();

    board.change_history(|history| {
        history.renew_task("10", &claude(), 5);
        history.post("11", "work", "10", 6, "still deploying");
    });

    assert!(until_reported(&mut following.states, &ReplicaState::Live { stored: 2 }).await);
    following.task.abort();
    let replica = Replica::open(&following.file).unwrap();
    assert_eq!(
        replica.task("10").unwrap().unwrap().claim,
        Some(Claim {
            actor: claude().id,
            until: lease_until(5),
        })
    );
}

#[tokio::test]
async fn an_actor_seen_in_the_feed_keeps_its_latest_details() {
    let mut history = BoardHistory::default();
    history.post_as(&claude(), "10", "work", "10", 0, "deploy www");
    let board = FakeBoard::with_history(history);
    let mut following = copy_then_follow(&board).await.unwrap();
    let renamed = claude().renamed("claude-opus");

    board.change_history(|history| {
        history.see_actor(&renamed, 1);
        history.post("11", "work", "10", 2, "who is that?");
    });

    assert!(until_reported(&mut following.states, &ReplicaState::Live { stored: 2 }).await);
    following.task.abort();
    let replica = Replica::open(&following.file).unwrap();
    assert_eq!(
        replica.actor(&claude().id).unwrap().map(|actor| actor.name),
        Some(String::from("claude-opus"))
    );
}

#[tokio::test]
async fn bookkeeping_that_concerns_no_message_is_read_and_skipped() {
    let mut history = history_across_two_channels();
    history.sync_thread("10", 6);
    let board = FakeBoard::with_history(history);
    let mut following = copy_then_follow(&board).await.unwrap();

    board.change_history(|history| {
        history.sync_thread("13", 7);
        history.post("17", "fleet/www", "13", 7, "green again");
    });

    assert!(until_reported(&mut following.states, &ReplicaState::Live { stored: 8 }).await);
    following.task.abort();
    let replica = Replica::open(&following.file).unwrap();
    assert_eq!(
        replica.feed_cursor().unwrap(),
        Some(String::from("event-13"))
    );
}

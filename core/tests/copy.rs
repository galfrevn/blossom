#[path = "support/history.rs"]
mod history;

use std::error::Error;

use blossom::contract::ReplicaState;
use blossom::replica::Replica;
use blossom::{BoardAccess, CopiedBoard, copy_board};
use history::{BoardHistory, serve_history};
use tempfile::TempDir;
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
    serve_history(&server, history.clone()).await;
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

    assert_eq!(copy.copied.replica.message_count().unwrap(), 7);
    for id in ["10", "11", "12", "13", "14", "15", "16"] {
        assert_eq!(
            copy.copied.replica.stored_view(id).unwrap().unwrap(),
            history.current_view(id)
        );
    }
}

#[tokio::test]
async fn the_copy_reports_its_progress_against_the_total_and_ends_copied() {
    let history = history_across_two_channels();

    let copy = copy_of(&history).await.unwrap();

    assert!(copy.reported_states.contains(&ReplicaState::Copying {
        stored: 7,
        total: Some(7)
    }));
    assert_eq!(
        copy.reported_states.last(),
        Some(&ReplicaState::Copied { stored: 7 })
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

    assert_eq!(copy.copied.replica.message_count().unwrap(), 2);
}

#[tokio::test]
async fn an_empty_board_is_copied_as_an_empty_replica() {
    let history = BoardHistory::default();

    let copy = copy_of(&history).await.unwrap();

    assert_eq!(copy.copied.replica.message_count().unwrap(), 0);
    assert_eq!(
        copy.reported_states.last(),
        Some(&ReplicaState::Copied { stored: 0 })
    );
}

mod support;

use blossom::contract::{OpenBoard, Person};
use blossom::find_open_board;
use support::{start_board_whose_discovery_names, start_fake_board, write_token_file};
use tempfile::TempDir;

fn ada() -> Person {
    Person {
        handle: String::from("ada"),
        name: String::from("Ada Lovelace"),
    }
}

#[tokio::test]
async fn the_running_local_board_opens_as_the_person_its_token_names() {
    let board = start_fake_board("token-of-ada", "ada", "Ada Lovelace").await;
    let token_folder = TempDir::new().unwrap();
    write_token_file(
        token_folder.path(),
        "board.json",
        &board.url,
        "token-of-ada",
        0,
    )
    .unwrap();

    let open_board = find_open_board(token_folder.path()).await;

    assert_eq!(
        open_board,
        OpenBoard::Open {
            url: board.url.clone(),
            person: ada()
        }
    );
}

#[tokio::test]
async fn no_board_is_running_when_the_token_folder_does_not_exist() {
    let parent_folder = TempDir::new().unwrap();

    let open_board = find_open_board(&parent_folder.path().join("local")).await;

    assert_eq!(open_board, OpenBoard::NoneRunning);
}

#[tokio::test]
async fn no_board_is_running_when_the_token_folder_is_empty() {
    let token_folder = TempDir::new().unwrap();

    let open_board = find_open_board(token_folder.path()).await;

    assert_eq!(open_board, OpenBoard::NoneRunning);
}

#[tokio::test]
async fn a_token_file_that_is_not_json_is_ignored() {
    let token_folder = TempDir::new().unwrap();
    std::fs::write(token_folder.path().join("board.json"), "not json").unwrap();

    let open_board = find_open_board(token_folder.path()).await;

    assert_eq!(open_board, OpenBoard::NoneRunning);
}

#[tokio::test]
async fn a_board_that_does_not_answer_is_not_opened() {
    let token_folder = TempDir::new().unwrap();
    let closed_port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let url_where_nothing_listens = format!("http://127.0.0.1:{closed_port}/api/board");
    write_token_file(
        token_folder.path(),
        "board.json",
        &url_where_nothing_listens,
        "token-of-ada",
        0,
    )
    .unwrap();

    let open_board = find_open_board(token_folder.path()).await;

    assert_eq!(open_board, OpenBoard::NoneRunning);
}

#[tokio::test]
async fn a_board_that_refuses_the_token_is_not_opened() {
    let board = start_fake_board("token-of-ada", "ada", "Ada Lovelace").await;
    let token_folder = TempDir::new().unwrap();
    write_token_file(
        token_folder.path(),
        "board.json",
        &board.url,
        "token-from-an-earlier-run",
        0,
    )
    .unwrap();

    let open_board = find_open_board(token_folder.path()).await;

    assert_eq!(open_board, OpenBoard::NoneRunning);
}

#[tokio::test]
async fn of_several_running_boards_the_one_started_most_recently_opens() {
    let board_started_an_hour_ago = start_fake_board("token-a", "ada", "Ada Lovelace").await;
    let board_started_a_minute_ago = start_fake_board("token-b", "ada", "Ada Lovelace").await;
    let board_started_a_day_ago = start_fake_board("token-c", "ada", "Ada Lovelace").await;
    let token_folder = TempDir::new().unwrap();
    write_token_file(
        token_folder.path(),
        "a.json",
        &board_started_an_hour_ago.url,
        "token-a",
        3_600,
    )
    .unwrap();
    write_token_file(
        token_folder.path(),
        "b.json",
        &board_started_a_minute_ago.url,
        "token-b",
        60,
    )
    .unwrap();
    write_token_file(
        token_folder.path(),
        "c.json",
        &board_started_a_day_ago.url,
        "token-c",
        86_400,
    )
    .unwrap();

    let open_board = find_open_board(token_folder.path()).await;

    assert_eq!(
        open_board,
        OpenBoard::Open {
            url: board_started_a_minute_ago.url.clone(),
            person: ada()
        }
    );
}

#[tokio::test]
async fn a_token_file_whose_url_is_not_on_loopback_is_ignored_without_a_request() {
    let board = start_fake_board("token-of-ada", "ada", "Ada Lovelace").await;
    let port = board.server.address().port();
    let url_not_on_loopback = format!("http://0.0.0.0:{port}/api/board");
    let token_folder = TempDir::new().unwrap();
    write_token_file(
        token_folder.path(),
        "board.json",
        &url_not_on_loopback,
        "token-of-ada",
        0,
    )
    .unwrap();

    let open_board = find_open_board(token_folder.path()).await;

    assert_eq!(open_board, OpenBoard::NoneRunning);
    assert_eq!(board.server.received_requests().await.unwrap().len(), 0);
}

#[tokio::test]
async fn a_board_whose_discovery_names_another_url_is_not_opened() {
    let board = start_board_whose_discovery_names(
        "http://127.0.0.1:4100/api/board",
        "token-of-ada",
        "ada",
        "Ada Lovelace",
    )
    .await;
    let token_folder = TempDir::new().unwrap();
    write_token_file(
        token_folder.path(),
        "board.json",
        &board.url,
        "token-of-ada",
        0,
    )
    .unwrap();

    let open_board = find_open_board(token_folder.path()).await;

    assert_eq!(open_board, OpenBoard::NoneRunning);
}

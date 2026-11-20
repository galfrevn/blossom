mod support;

use blossom::OpenBoardFollower;
use blossom::contract::{OpenBoard, Person};
use support::{answer_discovery_naming, answer_me, start_fake_board, write_token_file};
use tempfile::TempDir;
use wiremock::MockServer;

fn ada() -> Person {
    Person {
        handle: String::from("ada"),
        name: String::from("Ada Lovelace"),
    }
}

#[tokio::test]
async fn a_look_that_finds_nothing_changed_reports_nothing_and_asks_the_board_nothing() {
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
    let mut follower = OpenBoardFollower::new(token_folder.path().to_path_buf());
    follower.look().await;
    let requests_after_the_first_look = board.server.received_requests().await.unwrap().len();

    let second_look = follower.look().await;

    assert_eq!(second_look, None);
    assert_eq!(
        board.server.received_requests().await.unwrap().len(),
        requests_after_the_first_look
    );
}

#[tokio::test]
async fn a_board_that_starts_while_followed_opens_on_the_next_look() {
    let token_folder = TempDir::new().unwrap();
    let mut follower = OpenBoardFollower::new(token_folder.path().to_path_buf());
    assert_eq!(follower.look().await, Some(OpenBoard::NoneRunning));

    let board = start_fake_board("token-of-ada", "ada", "Ada Lovelace").await;
    write_token_file(
        token_folder.path(),
        "board.json",
        &board.url,
        "token-of-ada",
        0,
    )
    .unwrap();

    assert_eq!(
        follower.look().await,
        Some(OpenBoard::Open {
            url: board.url.clone(),
            person: ada()
        })
    );
}

#[tokio::test]
async fn the_open_board_stopping_is_noticed_on_the_next_look() {
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
    let mut follower = OpenBoardFollower::new(token_folder.path().to_path_buf());
    follower.look().await;

    std::fs::remove_file(token_folder.path().join("board.json")).unwrap();

    assert_eq!(follower.look().await, Some(OpenBoard::NoneRunning));
}

#[tokio::test]
async fn a_newer_board_that_starts_while_one_is_open_takes_over() {
    let older_board = start_fake_board("token-older", "ada", "Ada Lovelace").await;
    let newer_board = start_fake_board("token-newer", "ada", "Ada Lovelace").await;
    let token_folder = TempDir::new().unwrap();
    write_token_file(
        token_folder.path(),
        "older.json",
        &older_board.url,
        "token-older",
        600,
    )
    .unwrap();
    let mut follower = OpenBoardFollower::new(token_folder.path().to_path_buf());
    follower.look().await;

    write_token_file(
        token_folder.path(),
        "newer.json",
        &newer_board.url,
        "token-newer",
        0,
    )
    .unwrap();

    assert_eq!(
        follower.look().await,
        Some(OpenBoard::Open {
            url: newer_board.url.clone(),
            person: ada()
        })
    );
}

#[tokio::test]
async fn when_the_newer_board_stops_the_older_one_opens_again() {
    let older_board = start_fake_board("token-older", "ada", "Ada Lovelace").await;
    let newer_board = start_fake_board("token-newer", "ada", "Ada Lovelace").await;
    let token_folder = TempDir::new().unwrap();
    write_token_file(
        token_folder.path(),
        "older.json",
        &older_board.url,
        "token-older",
        600,
    )
    .unwrap();
    write_token_file(
        token_folder.path(),
        "newer.json",
        &newer_board.url,
        "token-newer",
        0,
    )
    .unwrap();
    let mut follower = OpenBoardFollower::new(token_folder.path().to_path_buf());
    follower.look().await;

    std::fs::remove_file(token_folder.path().join("newer.json")).unwrap();

    assert_eq!(
        follower.look().await,
        Some(OpenBoard::Open {
            url: older_board.url.clone(),
            person: ada()
        })
    );
}

#[tokio::test]
async fn a_board_that_did_not_answer_when_its_file_appeared_opens_once_it_answers() {
    let port_with_nothing_yet = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let url = format!("http://127.0.0.1:{port_with_nothing_yet}/api/board");
    let token_folder = TempDir::new().unwrap();
    write_token_file(token_folder.path(), "board.json", &url, "token-of-ada", 0).unwrap();
    let mut follower = OpenBoardFollower::new(token_folder.path().to_path_buf());
    assert_eq!(follower.look().await, Some(OpenBoard::NoneRunning));

    let listener = std::net::TcpListener::bind(("127.0.0.1", port_with_nothing_yet)).unwrap();
    let board = MockServer::builder().listener(listener).start().await;
    answer_discovery_naming(&board, &url).await;
    answer_me(&board, "token-of-ada", "ada", "Ada Lovelace").await;

    assert_eq!(
        follower.look().await,
        Some(OpenBoard::Open {
            url: url.clone(),
            person: ada()
        })
    );
}

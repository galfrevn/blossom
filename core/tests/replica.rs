use blossom::BoardAccess;
use blossom::replica::{MessageView, Replica, replica_file};
use serde_json::json;
use tempfile::TempDir;

fn view(id: &str, version: u64, text: &str) -> Result<MessageView, serde_json::Error> {
    let board_view = json!({
        "id": id,
        "channel": "fleet/www",
        "thread": format!("thread-of-{id}"),
        "createdAt": "2026-11-20T10:00:00.000Z",
        "version": version,
        "text": text
    });
    MessageView::parse(&board_view.to_string())
}

#[test]
fn a_newer_view_replaces_the_stored_one() {
    let folder = TempDir::new().unwrap();
    let mut replica = Replica::open(&folder.path().join("board.sqlite")).unwrap();

    replica
        .apply(&[view("812", 812, "deploy failed").unwrap()])
        .unwrap();
    replica
        .apply(&[view("812", 815, "deploy failed, rolled back").unwrap()])
        .unwrap();

    let stored = replica.stored_view("812").unwrap().unwrap();
    assert_eq!(stored["text"], "deploy failed, rolled back");
}

#[test]
fn an_older_view_never_replaces_a_newer_one_whatever_order_they_arrive_in() {
    let folder = TempDir::new().unwrap();
    let mut replica = Replica::open(&folder.path().join("board.sqlite")).unwrap();

    replica
        .apply(&[view("812", 815, "deploy failed, rolled back").unwrap()])
        .unwrap();
    replica
        .apply(&[view("812", 812, "deploy failed").unwrap()])
        .unwrap();

    let stored = replica.stored_view("812").unwrap().unwrap();
    assert_eq!(stored["text"], "deploy failed, rolled back");
}

#[test]
fn the_replica_counts_each_message_once_however_many_views_of_it_arrive() {
    let folder = TempDir::new().unwrap();
    let mut replica = Replica::open(&folder.path().join("board.sqlite")).unwrap();

    replica
        .apply(&[
            view("812", 812, "deploy failed").unwrap(),
            view("813", 813, "looking").unwrap(),
            view("812", 815, "deploy failed, rolled back").unwrap(),
        ])
        .unwrap();

    assert_eq!(replica.message_count(), 2);
}

#[test]
fn what_the_replica_holds_survives_reopening_it() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&[view("812", 812, "deploy failed").unwrap()])
        .unwrap();
    drop(replica);

    let reopened = Replica::open(&file).unwrap();

    assert_eq!(reopened.message_count(), 1);
    assert_eq!(
        reopened.stored_view("812").unwrap().unwrap()["text"],
        "deploy failed"
    );
}

#[test]
fn each_board_address_gets_a_replica_file_of_its_own_in_the_folder() {
    let folder = TempDir::new().unwrap();
    let first = BoardAccess::new("http://127.0.0.1:4100/api/board", "token-a");
    let second = BoardAccess::new("http://127.0.0.1:4200/api/board", "token-b");

    let first_file = replica_file(folder.path(), &first);

    assert_eq!(first_file.parent(), Some(folder.path()));
    assert_eq!(first_file, replica_file(folder.path(), &first));
    assert_ne!(first_file, replica_file(folder.path(), &second));
}

#[test]
fn the_feed_cursor_is_kept_with_the_views_it_came_with_across_reopening() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    assert_eq!(replica.feed_cursor().unwrap(), None);

    replica
        .apply_from_feed(&[view("812", 812, "deploy failed").unwrap()], "812")
        .unwrap();
    replica
        .apply_from_feed(&[view("813", 813, "looking").unwrap()], "815")
        .unwrap();
    drop(replica);
    let reopened = Replica::open(&file).unwrap();

    assert_eq!(reopened.feed_cursor().unwrap(), Some(String::from("815")));
    assert_eq!(reopened.message_count(), 2);
}

#[test]
fn the_count_takes_only_messages_it_has_not_held_before() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();

    replica
        .apply(&[
            view("812", 812, "deploy failed").unwrap(),
            view("813", 813, "looking").unwrap(),
            view("812", 815, "deploy failed, rolled back").unwrap(),
            view("813", 811, "an older look").unwrap(),
            view("812", 815, "deploy failed, rolled back").unwrap(),
        ])
        .unwrap();
    replica
        .apply_from_feed(&[view("813", 820, "found it").unwrap()], "820")
        .unwrap();
    replica
        .apply_from_feed(&[view("814", 821, "fixed").unwrap()], "821")
        .unwrap();
    let counted_while_open = replica.message_count();
    drop(replica);

    assert_eq!(counted_while_open, 3);
    assert_eq!(Replica::open(&file).unwrap().message_count(), 3);
}

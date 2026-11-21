use blossom::BoardAccess;
use blossom::replica::{
    BoardIdentity, MessageView, Replica, known_replica_files, new_replica_file, replica_file_for,
};
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

fn identity(oldest_id: &str, oldest_created_at: &str) -> BoardIdentity {
    BoardIdentity {
        oldest_id: String::from(oldest_id),
        oldest_created_at: String::from(oldest_created_at),
    }
}

#[test]
fn a_replica_file_is_named_after_the_boards_address_and_oldest_message() {
    let folder = TempDir::new().unwrap();
    let port_4100 = BoardAccess::new("http://127.0.0.1:4100/api/board", "token-a");
    let port_4200 = BoardAccess::new("http://127.0.0.1:4200/api/board", "token-b");
    let first_project = identity("10", "2026-11-20T10:00:00.000Z");
    let second_project = identity("10", "2026-11-20T10:40:00.000Z");

    let file = replica_file_for(folder.path(), &port_4100, &first_project);

    assert_eq!(file.parent(), Some(folder.path()));
    assert_eq!(
        file,
        replica_file_for(folder.path(), &port_4100, &first_project)
    );
    assert_ne!(
        file,
        replica_file_for(folder.path(), &port_4100, &second_project)
    );
    assert_ne!(
        file,
        replica_file_for(folder.path(), &port_4200, &first_project)
    );
    assert_ne!(file, new_replica_file(folder.path(), &port_4100));
}

#[test]
fn the_known_replicas_of_an_address_are_its_finished_copies_only() {
    let folder = TempDir::new().unwrap();
    let port_4100 = BoardAccess::new("http://127.0.0.1:4100/api/board", "token-a");
    let port_4200 = BoardAccess::new("http://127.0.0.1:4200/api/board", "token-b");
    let first_project = identity("10", "2026-11-20T10:00:00.000Z");
    let second_project = identity("10", "2026-11-20T10:40:00.000Z");
    for file in [
        replica_file_for(folder.path(), &port_4100, &first_project),
        replica_file_for(folder.path(), &port_4100, &second_project),
        replica_file_for(folder.path(), &port_4200, &first_project),
        new_replica_file(folder.path(), &port_4100),
    ] {
        Replica::open(&file).unwrap();
    }

    let mut known = known_replica_files(folder.path(), &port_4100);
    known.sort();

    let mut expected = vec![
        replica_file_for(folder.path(), &port_4100, &first_project),
        replica_file_for(folder.path(), &port_4100, &second_project),
    ];
    expected.sort();
    assert_eq!(known, expected);
}

#[test]
fn the_boards_identity_is_kept_across_reopening() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    assert_eq!(replica.identity().unwrap(), None);

    replica
        .keep_identity(&identity("10", "2026-11-20T10:00:00.000Z"))
        .unwrap();
    drop(replica);

    assert_eq!(
        Replica::open(&file).unwrap().identity().unwrap(),
        Some(identity("10", "2026-11-20T10:00:00.000Z"))
    );
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

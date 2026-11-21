use blossom::reader::ReplicaReader;
use blossom::replica::{MessageView, Replica};
use rusqlite::Connection;
use serde_json::json;
use tempfile::TempDir;

fn view_in(channel: &str, id: &str) -> Result<MessageView, serde_json::Error> {
    let board_view = json!({
        "id": id,
        "channel": channel,
        "thread": format!("thread-of-{id}"),
        "createdAt": "2026-11-21T10:00:00.000Z",
        "version": 1,
        "text": "deploy"
    });
    MessageView::parse(&board_view.to_string())
}

#[test]
fn the_channels_are_every_channel_the_replica_holds_a_message_in() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&[
            view_in("fleet/www", "1").unwrap(),
            view_in("general", "2").unwrap(),
            view_in("fleet/api", "3").unwrap(),
            view_in("fleet/www", "4").unwrap(),
        ])
        .unwrap();

    let reader = ReplicaReader::open(&file).unwrap();

    assert_eq!(
        reader.channels().unwrap(),
        vec!["fleet/api", "fleet/www", "general"]
    );
}

#[test]
fn a_channel_appears_to_the_reader_once_a_message_arrives_in_it() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica.apply(&[view_in("general", "1").unwrap()]).unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    replica
        .apply(&[view_in("fleet/docs", "2").unwrap()])
        .unwrap();

    assert_eq!(reader.channels().unwrap(), vec!["fleet/docs", "general"]);
}

#[test]
fn a_replica_copied_before_channels_were_kept_still_lists_them() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let older = Connection::open(&file).unwrap();
    older
        .execute_batch(
            "CREATE TABLE messages (
                 id TEXT PRIMARY KEY,
                 channel TEXT NOT NULL,
                 thread TEXT NOT NULL,
                 created_at TEXT NOT NULL,
                 version INTEGER NOT NULL,
                 view TEXT NOT NULL
             ) WITHOUT ROWID;
             INSERT INTO messages VALUES ('1', 'fleet/www', 't', '2026-11-21T10:00:00.000Z', 1, '{}');",
        )
        .unwrap();
    drop(older);

    Replica::open(&file).unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    assert_eq!(reader.channels().unwrap(), vec!["fleet/www"]);
}

#[test]
fn the_reader_never_writes_to_the_replica() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    Replica::open(&file).unwrap();

    let reader = ReplicaReader::open(&file).unwrap();

    assert!(reader.is_read_only());
}

fn view_at(
    id: &str,
    minute: u32,
    second: u32,
    text: &str,
) -> Result<MessageView, serde_json::Error> {
    let board_view = json!({
        "id": id,
        "channel": "fleet/www",
        "thread": format!("thread-of-{id}"),
        "createdAt": format!("2026-11-21T10:{minute:02}:{second:02}.000Z"),
        "version": 1,
        "text": text,
        "author": { "kind": "agent", "handle": "ada/claude", "name": "claude", "via": { "name": "ada", "environment": null } }
    });
    MessageView::parse(&board_view.to_string())
}

fn ids_of(window: &blossom::contract::FlowWindow) -> Vec<String> {
    window.rows.iter().map(|row| row.id.clone()).collect()
}

#[test]
fn the_flow_is_every_message_oldest_first() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&[
            view_at("3", 2, 0, "third").unwrap(),
            view_at("1", 0, 0, "first").unwrap(),
            view_at("2", 1, 0, "second").unwrap(),
        ])
        .unwrap();

    let reader = ReplicaReader::open(&file).unwrap();
    let window = reader.flow_window(0, 10).unwrap();

    assert_eq!(window.total, 3);
    assert_eq!(window.start, 0);
    assert_eq!(ids_of(&window), vec!["1", "2", "3"]);
}

#[test]
fn a_window_is_read_at_any_depth() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    let views: Vec<MessageView> = (0..300)
        .map(|index| view_at(&format!("{index:03}"), index / 60, index % 60, "work").unwrap())
        .collect();
    replica.apply(&views).unwrap();

    let reader = ReplicaReader::open(&file).unwrap();
    let window = reader.flow_window(250, 3).unwrap();
    let past_the_end = reader.flow_window(299, 10).unwrap();

    assert_eq!(ids_of(&window), vec!["250", "251", "252"]);
    assert_eq!(ids_of(&past_the_end), vec!["299"]);
    assert_eq!(past_the_end.total, 300);
}

#[test]
fn a_message_that_arrives_takes_its_place_once_the_reader_catches_up() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&[
            view_at("1", 0, 0, "first").unwrap(),
            view_at("3", 2, 0, "third").unwrap(),
        ])
        .unwrap();
    let mut reader = ReplicaReader::open(&file).unwrap();

    replica
        .apply(&[
            view_at("4", 3, 0, "newest").unwrap(),
            view_at("2", 1, 0, "late").unwrap(),
        ])
        .unwrap();
    let caught_up = reader.catch_up().unwrap();

    assert!(caught_up);
    assert_eq!(
        ids_of(&reader.flow_window(0, 10).unwrap()),
        vec!["1", "2", "3", "4"]
    );
    assert!(!reader.catch_up().unwrap());
}

#[test]
fn an_edited_message_reads_as_it_stands_now() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&[view_at("1", 0, 0, "deploy failed").unwrap()])
        .unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    let edited = json!({
        "id": "1", "channel": "fleet/www", "thread": "thread-of-1",
        "createdAt": "2026-11-21T10:00:00.000Z", "version": 2,
        "text": "deploy failed, rolled back", "editedAt": "2026-11-21T10:05:00.000Z"
    });
    replica
        .apply(&[MessageView::parse(&edited.to_string()).unwrap()])
        .unwrap();
    let row = reader.flow_window(0, 1).unwrap().rows.remove(0);

    assert_eq!(row.text, "deploy failed, rolled back");
    assert!(row.edited);
}

#[test]
fn a_row_says_who_wrote_it_and_for_whom() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&[view_at("1", 0, 0, "on it").unwrap()])
        .unwrap();

    let row = ReplicaReader::open(&file)
        .unwrap()
        .flow_window(0, 1)
        .unwrap()
        .rows
        .remove(0);

    assert_eq!(row.author, "claude");
    assert_eq!(row.on_behalf_of.as_deref(), Some("ada"));
    assert_eq!(row.channel, "fleet/www");
    assert_eq!(row.created_at, "2026-11-21T10:00:00.000Z");
}

#[test]
fn the_first_message_of_each_minute_starts_it() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&[
            view_at("1", 0, 5, "a").unwrap(),
            view_at("2", 0, 40, "b").unwrap(),
            view_at("3", 1, 2, "c").unwrap(),
        ])
        .unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    let whole: Vec<bool> = reader
        .flow_window(0, 3)
        .unwrap()
        .rows
        .iter()
        .map(|row| row.first_of_its_minute)
        .collect();
    let from_the_middle = reader.flow_window(1, 2).unwrap().rows.remove(0);

    assert_eq!(whole, vec![true, false, true]);
    assert!(!from_the_middle.first_of_its_minute);
}

#[test]
fn a_deleted_message_reads_as_deleted() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    let deleted = json!({
        "id": "1", "channel": "fleet/www", "thread": "thread-of-1",
        "createdAt": "2026-11-21T10:00:00.000Z", "version": 3, "text": "", "deleted": true
    });
    replica
        .apply(&[MessageView::parse(&deleted.to_string()).unwrap()])
        .unwrap();

    let row = ReplicaReader::open(&file)
        .unwrap()
        .flow_window(0, 1)
        .unwrap()
        .rows
        .remove(0);

    assert!(row.deleted);
}

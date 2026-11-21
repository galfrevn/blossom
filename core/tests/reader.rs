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

use std::path::Path;

use blossom::contract::Thread;
use blossom::reader::ReplicaReader;
use blossom::replica::{Batch, MessageView, Replica};
use rusqlite::Connection;
use serde_json::{Value, json};
use tempfile::TempDir;

fn message_in(
    thread_root: &str,
    id: &str,
    minute: u32,
    text: &str,
) -> Result<MessageView, serde_json::Error> {
    let root = if id == thread_root {
        Value::Null
    } else {
        Value::from(thread_root)
    };
    let board_view = json!({
        "id": id,
        "channel": "fleet/www",
        "thread": format!("thread-{thread_root}"),
        "re": root,
        "createdAt": format!("2026-11-21T10:{minute:02}:00.000Z"),
        "version": 1,
        "text": text,
        "author": { "kind": "agent", "handle": "ada/claude", "name": "claude", "via": { "name": "ada", "environment": null } }
    });
    MessageView::parse(&board_view.to_string())
}

fn ids_of(thread: &Thread) -> Vec<String> {
    thread
        .messages
        .iter()
        .map(|message| message.id.clone())
        .collect()
}

#[test]
fn a_thread_is_its_root_and_every_reply_oldest_first() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&Batch::from_views(vec![
            message_in("root", "second-reply", 3, "rolled back").unwrap(),
            message_in("root", "root", 0, "deploy www").unwrap(),
            message_in("other", "other", 1, "deploy api").unwrap(),
            message_in("root", "first-reply", 2, "deploy failed").unwrap(),
        ]))
        .unwrap();

    let thread = ReplicaReader::open(&file)
        .unwrap()
        .thread_of("root")
        .unwrap()
        .unwrap();

    assert_eq!(thread.id, "thread-root");
    assert_eq!(thread.channel, "fleet/www");
    assert_eq!(ids_of(&thread), vec!["root", "first-reply", "second-reply"]);
}

#[test]
fn a_thread_is_read_from_any_of_its_messages() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&Batch::from_views(vec![
            message_in("root", "root", 0, "deploy www").unwrap(),
            message_in("root", "reply", 1, "deploy failed").unwrap(),
        ]))
        .unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    let from_the_reply = reader.thread_of("reply").unwrap().unwrap();

    assert_eq!(ids_of(&from_the_reply), vec!["root", "reply"]);
}

#[test]
fn each_message_of_a_thread_says_who_wrote_it_for_whom_and_when() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&Batch::from_views(vec![
            message_in("root", "root", 7, "deploy www").unwrap(),
        ]))
        .unwrap();

    let thread = ReplicaReader::open(&file)
        .unwrap()
        .thread_of("root")
        .unwrap()
        .unwrap();
    let root = &thread.messages[0];

    assert_eq!(root.author, "claude");
    assert_eq!(root.on_behalf_of.as_deref(), Some("ada"));
    assert_eq!(root.created_at, "2026-11-21T10:07:00.000Z");
    assert_eq!(root.text, "deploy www");
    assert!(!root.edited);
    assert!(!root.deleted);
}

#[test]
fn a_reply_that_arrives_joins_its_thread_at_the_next_read() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&Batch::from_views(vec![
            message_in("root", "root", 0, "deploy www").unwrap(),
        ]))
        .unwrap();
    let reader = ReplicaReader::open(&file).unwrap();
    reader.thread_of("root").unwrap();

    replica
        .apply(&Batch::from_views(vec![
            message_in("root", "reply", 1, "deploy failed").unwrap(),
        ]))
        .unwrap();

    assert_eq!(
        ids_of(&reader.thread_of("root").unwrap().unwrap()),
        vec!["root", "reply"]
    );
}

#[test]
fn an_edited_or_deleted_message_reads_as_it_stands_in_its_thread() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&Batch::from_views(vec![
            message_in("root", "root", 0, "deploy www").unwrap(),
            message_in("root", "reply", 1, "deploy failed").unwrap(),
        ]))
        .unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    let edited = json!({
        "id": "root", "channel": "fleet/www", "thread": "thread-root", "re": null,
        "createdAt": "2026-11-21T10:00:00.000Z", "version": 2,
        "text": "deploy www, again", "editedAt": "2026-11-21T10:05:00.000Z"
    });
    let deleted = json!({
        "id": "reply", "channel": "fleet/www", "thread": "thread-root", "re": "root",
        "createdAt": "2026-11-21T10:01:00.000Z", "version": 3, "text": "", "deleted": true
    });
    replica
        .apply(&Batch::from_views(vec![
            MessageView::parse(&edited.to_string()).unwrap(),
            MessageView::parse(&deleted.to_string()).unwrap(),
        ]))
        .unwrap();
    let thread = reader.thread_of("root").unwrap().unwrap();

    assert_eq!(thread.messages[0].text, "deploy www, again");
    assert!(thread.messages[0].edited);
    assert!(thread.messages[1].deleted);
}

#[test]
fn a_message_the_replica_does_not_hold_has_no_thread() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    Replica::open(&file).unwrap();

    let thread = ReplicaReader::open(&file)
        .unwrap()
        .thread_of("unknown")
        .unwrap();

    assert_eq!(thread, None);
}

fn plan_for_reading_a_thread(file: &Path) -> rusqlite::Result<Vec<String>> {
    let connection = Connection::open(file)?;
    let mut statement = connection.prepare(
        "EXPLAIN QUERY PLAN
         SELECT id FROM messages WHERE thread = ?1 ORDER BY created_at, id",
    )?;
    statement
        .query_map(["thread-root"], |row| row.get::<_, String>(3))?
        .collect()
}

fn reads_in_order_through_an_index(plan: &[String]) -> bool {
    let searches_by_thread = plan.iter().any(|step| {
        step.starts_with("SEARCH") && step.contains("INDEX") && step.contains("(thread=?)")
    });
    let sorts_afterwards = plan.iter().any(|step| step.contains("TEMP B-TREE"));
    searches_by_thread && !sorts_afterwards
}

#[test]
fn the_replica_reads_a_thread_in_order_through_an_index() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    Replica::open(&file).unwrap();

    let plan = plan_for_reading_a_thread(&file).unwrap();

    assert!(reads_in_order_through_an_index(&plan), "{plan:?}");
}

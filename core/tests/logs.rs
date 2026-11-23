use std::error::Error;
use std::path::{Path, PathBuf};

use blossom::contract::{FlowWindow, TaskCount, TaskNarrowing, TaskWindow};
use blossom::reader::ReplicaReader;
use blossom::replica::{Batch, MessageView, Replica, ReplicaError};
use serde_json::{Value, json};
use tempfile::TempDir;

const ADA: &str = "local:user:ada";

fn at_second(second: u32) -> String {
    format!("2026-11-23T10:00:{second:02}.000Z")
}

fn task_in(state: &str) -> Value {
    json!({
        "state": state,
        "owner": ADA,
        "by": ADA,
        "for": null,
        "needs": [],
        "epoch": 1,
        "claim": null,
        "blocked": null,
        "result": null,
        "reason": null
    })
}

fn root(id: &str, second: u32, version: u64, task: Value) -> Value {
    json!({
        "id": id,
        "channel": "fleet/www",
        "thread": format!("thread-of-{id}"),
        "re": null,
        "createdAt": at_second(second),
        "version": version,
        "text": format!("task {id}"),
        "by": { "actor": ADA, "principal": ADA, "profile": null },
        "author": { "kind": "person", "handle": "ada", "name": "Ada Lovelace" },
        "task": task
    })
}

fn reply(id: &str, root: &str, second: u32, version: u64, text: &str) -> Value {
    json!({
        "id": id,
        "channel": "fleet/www",
        "thread": format!("thread-of-{root}"),
        "re": root,
        "createdAt": at_second(second),
        "version": version,
        "text": text,
        "attachments": [],
        "author": { "kind": "agent", "handle": "ada/claude", "name": "claude", "via": { "name": "ada", "environment": null } }
    })
}

fn progress(id: &str, root: &str, second: u32) -> Value {
    reply(id, root, second, 1, "still running the tests")
}

fn batch_of(views: &[Value]) -> Result<Batch, serde_json::Error> {
    let views = views
        .iter()
        .map(|view| MessageView::parse(&view.to_string()))
        .collect::<Result<Vec<MessageView>, serde_json::Error>>()?;
    Ok(Batch::from_views(views))
}

fn from_the_feed(replica: &mut Replica, views: &[Value]) -> Result<(), Box<dyn Error>> {
    replica.apply_from_feed(&batch_of(views)?, "event-latest")?;
    Ok(())
}

fn replica_in(folder: &TempDir) -> Result<(Replica, PathBuf), ReplicaError> {
    let file = folder.path().join("board.sqlite");
    Ok((Replica::open(&file)?, file))
}

fn rows_in(file: &Path, log: &str) -> rusqlite::Result<i64> {
    let connection = rusqlite::Connection::open(file)?;
    connection.query_row(&format!("SELECT count(*) FROM {log}"), [], |row| row.get(0))
}

fn a_thread_with_progress() -> Vec<Value> {
    vec![
        root("10", 0, 1, Value::Null),
        progress("11", "10", 1),
        progress("12", "10", 2),
    ]
}

fn the_thread_made_a_task() -> Value {
    root("10", 0, 2, task_in("ready"))
}

fn catch_up(reader: &mut ReplicaReader) -> Result<(), ReplicaError> {
    reader.catch_up()?;
    reader.catch_up_tasks()?;
    Ok(())
}

#[test]
fn the_feed_s_next_batch_deletes_the_rows_every_open_reader_has_read() {
    let folder = TempDir::new().unwrap();
    let (mut replica, file) = replica_in(&folder).unwrap();
    replica
        .apply(&batch_of(&a_thread_with_progress()).unwrap())
        .unwrap();
    from_the_feed(&mut replica, &[the_thread_made_a_task()]).unwrap();
    let mut reader = ReplicaReader::open(&file).unwrap();
    assert_eq!(rows_in(&file, "task_changes").unwrap(), 1);
    assert_eq!(rows_in(&file, "message_changes").unwrap(), 2);

    from_the_feed(&mut replica, &[root("10", 0, 3, task_in("claimed"))]).unwrap();
    assert_eq!(rows_in(&file, "task_changes").unwrap(), 1);
    assert_eq!(rows_in(&file, "message_changes").unwrap(), 0);
    catch_up(&mut reader).unwrap();
    from_the_feed(&mut replica, &[]).unwrap();

    assert_eq!(rows_in(&file, "task_changes").unwrap(), 0);
    assert_eq!(rows_in(&file, "arrivals").unwrap(), 3);
}

#[test]
fn rows_a_reader_has_not_read_are_kept_though_another_has_read_them() {
    let folder = TempDir::new().unwrap();
    let (mut replica, file) = replica_in(&folder).unwrap();
    replica
        .apply(&batch_of(&a_thread_with_progress()).unwrap())
        .unwrap();
    let mut ahead = ReplicaReader::open(&file).unwrap();
    let mut behind = ReplicaReader::open(&file).unwrap();

    from_the_feed(&mut replica, &[the_thread_made_a_task()]).unwrap();
    catch_up(&mut ahead).unwrap();
    from_the_feed(&mut replica, &[]).unwrap();
    assert_eq!(rows_in(&file, "task_changes").unwrap(), 1);
    assert_eq!(rows_in(&file, "message_changes").unwrap(), 2);

    catch_up(&mut behind).unwrap();
    from_the_feed(&mut replica, &[]).unwrap();

    assert_eq!(rows_in(&file, "task_changes").unwrap(), 0);
    assert_eq!(rows_in(&file, "message_changes").unwrap(), 0);
    assert_eq!(
        behind.flow_window_in(None, 0, 10).unwrap(),
        ahead.flow_window_in(None, 0, 10).unwrap()
    );
}

#[test]
fn nothing_is_deleted_while_no_reader_is_open() {
    let folder = TempDir::new().unwrap();
    let (mut replica, file) = replica_in(&folder).unwrap();
    replica
        .apply(&batch_of(&a_thread_with_progress()).unwrap())
        .unwrap();
    from_the_feed(&mut replica, &[the_thread_made_a_task()]).unwrap();

    let mut closed = ReplicaReader::open(&file).unwrap();
    catch_up(&mut closed).unwrap();
    drop(closed);
    from_the_feed(&mut replica, &[root("10", 0, 3, task_in("claimed"))]).unwrap();

    assert_eq!(rows_in(&file, "task_changes").unwrap(), 2);
    assert_eq!(rows_in(&file, "message_changes").unwrap(), 2);
}

#[test]
fn a_copy_s_batch_deletes_nothing() {
    let folder = TempDir::new().unwrap();
    let (mut replica, file) = replica_in(&folder).unwrap();
    replica
        .apply(&batch_of(&a_thread_with_progress()).unwrap())
        .unwrap();
    replica
        .apply(&batch_of(&[the_thread_made_a_task()]).unwrap())
        .unwrap();
    let _reader = ReplicaReader::open(&file).unwrap();

    replica
        .apply(&batch_of(&[root("10", 0, 3, task_in("claimed"))]).unwrap())
        .unwrap();

    assert_eq!(rows_in(&file, "task_changes").unwrap(), 2);
    assert_eq!(rows_in(&file, "message_changes").unwrap(), 2);
}

#[test]
fn a_change_stored_after_every_row_was_deleted_is_still_read() {
    let folder = TempDir::new().unwrap();
    let (mut replica, file) = replica_in(&folder).unwrap();
    replica
        .apply(
            &batch_of(&[
                root("10", 0, 1, task_in("ready")),
                progress("11", "10", 1),
                progress("12", "10", 2),
            ])
            .unwrap(),
        )
        .unwrap();
    let mut reader = ReplicaReader::open(&file).unwrap();
    from_the_feed(&mut replica, &[]).unwrap();
    assert_eq!(rows_in(&file, "task_changes").unwrap(), 0);

    from_the_feed(
        &mut replica,
        &[
            root("10", 0, 2, task_in("blocked")),
            reply("12", "10", 2, 2, "over to @[local:user:ada]"),
        ],
    )
    .unwrap();
    let flow_moved = reader.catch_up().unwrap();
    let tasks_moved = reader.catch_up_tasks().unwrap();

    assert!(flow_moved);
    assert!(tasks_moved);
    assert_eq!(
        reader.task_counts_in(None).first(),
        Some(&TaskCount {
            state: String::from("blocked"),
            count: 1
        })
    );
}

#[test]
fn a_change_stored_as_the_writer_deletes_is_read_on_the_next_catch_up() {
    let folder = TempDir::new().unwrap();
    let (mut replica, file) = replica_in(&folder).unwrap();
    replica
        .apply(&batch_of(&[root("10", 0, 1, task_in("ready"))]).unwrap())
        .unwrap();
    let mut reader = ReplicaReader::open(&file).unwrap();

    from_the_feed(&mut replica, &[root("10", 0, 2, task_in("claimed"))]).unwrap();
    assert_eq!(rows_in(&file, "task_changes").unwrap(), 1);

    assert!(reader.catch_up_tasks().unwrap());
    assert_eq!(
        reader.task_counts_in(None).get(2).map(|count| count.count),
        Some(1)
    );
}

struct Read {
    flow: FlowWindow,
    tasks: TaskWindow,
    counts: Vec<TaskCount>,
}

fn read_everything(reader: &ReplicaReader) -> Result<Read, ReplicaError> {
    Ok(Read {
        flow: reader.flow_window_in(None, 0, 100)?,
        tasks: reader.task_window_in(&TaskNarrowing::default(), 0, 100)?,
        counts: reader.task_counts_in(None),
    })
}

fn the_same_story_told_to(
    replica: &mut Replica,
    mut reader: Option<&mut ReplicaReader>,
) -> Result<(), Box<dyn Error>> {
    let chapters = [
        vec![the_thread_made_a_task(), progress("13", "10", 3)],
        vec![root("20", 4, 1, task_in("ready")), progress("21", "20", 5)],
        vec![reply("12", "10", 2, 2, "over to @[local:user:ada]")],
        vec![
            root("20", 4, 2, task_in("claimed")),
            root("10", 0, 3, task_in("blocked")),
        ],
        vec![
            root("30", 6, 1, Value::Null),
            progress("31", "30", 7),
            root("30", 6, 2, task_in("ready")),
        ],
    ];
    for chapter in chapters {
        from_the_feed(replica, &chapter)?;
        if let Some(reader) = reader.as_deref_mut() {
            catch_up(reader)?;
        }
    }
    Ok(())
}

#[test]
fn a_reader_that_opens_after_the_logs_were_trimmed_reads_what_one_that_never_trimmed_reads() {
    let trimmed_folder = TempDir::new().unwrap();
    let untrimmed_folder = TempDir::new().unwrap();
    let (mut trimmed, trimmed_file) = replica_in(&trimmed_folder).unwrap();
    let (mut untrimmed, untrimmed_file) = replica_in(&untrimmed_folder).unwrap();
    for replica in [&mut trimmed, &mut untrimmed] {
        replica
            .apply(&batch_of(&a_thread_with_progress()).unwrap())
            .unwrap();
    }
    let mut following = ReplicaReader::open(&trimmed_file).unwrap();

    the_same_story_told_to(&mut trimmed, Some(&mut following)).unwrap();
    the_same_story_told_to(&mut untrimmed, None).unwrap();
    from_the_feed(&mut trimmed, &[]).unwrap();

    assert_eq!(rows_in(&trimmed_file, "task_changes").unwrap(), 0);
    assert_eq!(rows_in(&trimmed_file, "message_changes").unwrap(), 0);
    let opened_after_the_trim =
        read_everything(&ReplicaReader::open(&trimmed_file).unwrap()).unwrap();
    let never_trimmed = read_everything(&ReplicaReader::open(&untrimmed_file).unwrap()).unwrap();
    let followed_throughout = read_everything(&following).unwrap();
    assert_eq!(opened_after_the_trim.flow, never_trimmed.flow);
    assert_eq!(opened_after_the_trim.tasks, never_trimmed.tasks);
    assert_eq!(opened_after_the_trim.counts, never_trimmed.counts);
    assert_eq!(followed_throughout.flow, never_trimmed.flow);
    assert_eq!(followed_throughout.tasks, never_trimmed.tasks);
}

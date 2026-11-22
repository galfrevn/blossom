use std::error::Error;
use std::path::{Path, PathBuf};

use blossom::contract::{Lease, TaskCount, TaskNarrowing, TaskRow, TaskWindow};
use blossom::reader::ReplicaReader;
use blossom::replica::{Batch, MessageView, Replica, ReplicaError};
use serde_json::{Value, json};
use tempfile::TempDir;

const ADA: &str = "local:user:ada";
const CLAUDE: &str = "local:user:ada+claude";
const CODEX: &str = "local:user:ada+codex";

fn time_at(minute: u32) -> String {
    format!("2026-11-21T10:{minute:02}:00.000Z")
}

fn ada() -> Value {
    json!({ "kind": "person", "handle": "ada", "name": "Ada Lovelace", "avatar": null })
}

fn agent_named(name: &str) -> Value {
    json!({
        "kind": "agent",
        "handle": format!("ada/{name}"),
        "name": name,
        "avatar": null,
        "via": { "name": "ada", "environment": null }
    })
}

fn listed(id: &str, details: Value) -> Value {
    let mut actor = details;
    actor["id"] = Value::from(id);
    actor
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

fn root(id: &str, channel: &str, minute: u32, text: &str, task: Value) -> Value {
    json!({
        "id": id,
        "channel": channel,
        "thread": format!("thread-of-{id}"),
        "re": null,
        "createdAt": time_at(minute),
        "version": 1,
        "text": text,
        "by": { "actor": ADA, "principal": ADA, "profile": null },
        "author": ada(),
        "task": task
    })
}

fn newer(view: &Value, version: u64) -> Value {
    let mut newer = view.clone();
    newer["version"] = Value::from(version);
    newer
}

fn event(
    cursor: &str,
    event_type: &str,
    minute: u32,
    actor: (&str, Value),
    payload: Value,
    message: &Value,
) -> Value {
    let (actor_id, details) = actor;
    let mut payload = payload;
    payload["id"] = message["id"].clone();
    payload["by"] = json!({ "actor": actor_id, "principal": ADA });
    json!({
        "cursor": cursor,
        "type": event_type,
        "createdAt": time_at(minute),
        "payload": payload,
        "message": message,
        "actor": listed(actor_id, details)
    })
}

fn store_roots(replica: &mut Replica, roots: &[Value]) -> Result<(), Box<dyn Error>> {
    let views = roots
        .iter()
        .map(|root| MessageView::parse(&root.to_string()))
        .collect::<Result<Vec<MessageView>, serde_json::Error>>()?;
    replica.apply(&Batch::from_views(views))?;
    Ok(())
}

fn store_events(replica: &mut Replica, events: &[Value]) -> Result<(), Box<dyn Error>> {
    let mut batch = Batch::default();
    for event in events {
        batch.add_event(&event.to_string())?;
    }
    replica.apply(&batch)?;
    Ok(())
}

fn replica_in(folder: &TempDir) -> Result<(Replica, PathBuf), ReplicaError> {
    let file = folder.path().join("board.sqlite");
    Ok((Replica::open(&file)?, file))
}

fn reader_of(file: &Path) -> Result<ReplicaReader, ReplicaError> {
    ReplicaReader::open(file)
}

fn everything() -> TaskNarrowing {
    TaskNarrowing::default()
}

fn in_channel(channel: &str) -> TaskNarrowing {
    TaskNarrowing {
        channel: Some(channel.to_owned()),
        ..TaskNarrowing::default()
    }
}

fn in_state(state: &str) -> TaskNarrowing {
    TaskNarrowing {
        state: Some(state.to_owned()),
        ..TaskNarrowing::default()
    }
}

fn titled(title: &str) -> TaskNarrowing {
    TaskNarrowing {
        title: title.to_owned(),
        ..TaskNarrowing::default()
    }
}

fn ids_of(window: &TaskWindow) -> Vec<String> {
    window.rows.iter().map(|row| row.id.clone()).collect()
}

fn all_ids(reader: &ReplicaReader, narrowing: &TaskNarrowing) -> Result<Vec<String>, ReplicaError> {
    Ok(ids_of(&reader.task_window_in(narrowing, 0, 1000)?))
}

fn only_row(reader: &ReplicaReader, id: &str) -> Result<Option<TaskRow>, ReplicaError> {
    let window = reader.task_window_in(&everything(), 0, 1000)?;
    Ok(window.rows.into_iter().find(|row| row.id == id))
}

fn count(state: &str, count: u32) -> TaskCount {
    TaskCount {
        state: state.to_owned(),
        count,
    }
}

#[test]
fn the_tasks_are_listed_open_ones_first_in_the_order_of_their_states() {
    let folder = TempDir::new().unwrap();
    let (mut replica, file) = replica_in(&folder).unwrap();
    let states = [
        "canceled",
        "waiting",
        "completed",
        "claimed",
        "ready",
        "failed",
        "blocked",
    ];
    let roots: Vec<Value> = states
        .iter()
        .map(|state| root(state, "work", 0, state, task_in(state)))
        .collect();
    store_roots(&mut replica, &roots).unwrap();

    let reader = reader_of(&file).unwrap();

    assert_eq!(
        all_ids(&reader, &everything()).unwrap(),
        vec![
            "blocked",
            "failed",
            "claimed",
            "ready",
            "waiting",
            "completed",
            "canceled"
        ]
    );
}

#[test]
fn within_a_state_the_task_that_changed_last_comes_first() {
    let folder = TempDir::new().unwrap();
    let (mut replica, file) = replica_in(&folder).unwrap();
    let quiet_since_it_was_made = root("old", "work", 0, "old", task_in("ready"));
    store_roots(
        &mut replica,
        &[
            root("first", "work", 1, "first", task_in("ready")),
            root("third", "work", 3, "third", task_in("ready")),
            root("second", "work", 2, "second", task_in("ready")),
            quiet_since_it_was_made.clone(),
        ],
    )
    .unwrap();
    store_events(
        &mut replica,
        &[event(
            "event-1",
            "task.unblocked",
            9,
            (ADA, ada()),
            json!({}),
            &newer(&quiet_since_it_was_made, 2),
        )],
    )
    .unwrap();

    let reader = reader_of(&file).unwrap();

    assert_eq!(
        all_ids(&reader, &everything()).unwrap(),
        vec!["old", "third", "second", "first"]
    );
    assert_eq!(
        only_row(&reader, "old").unwrap().unwrap().changed_at,
        time_at(9)
    );
    assert_eq!(
        only_row(&reader, "first").unwrap().unwrap().changed_at,
        time_at(1)
    );
}

#[test]
fn a_window_of_tasks_is_read_at_any_depth() {
    let folder = TempDir::new().unwrap();
    let (mut replica, file) = replica_in(&folder).unwrap();
    let roots: Vec<Value> = (0..300)
        .map(|index| {
            let id = format!("{index:03}");
            root(&id, "work", 59 - index / 6, &id, task_in("ready"))
        })
        .collect();
    store_roots(&mut replica, &roots).unwrap();

    let reader = reader_of(&file).unwrap();
    let window = reader.task_window_in(&everything(), 250, 3).unwrap();
    let past_the_end = reader.task_window_in(&everything(), 299, 10).unwrap();

    assert_eq!(window.total, 300);
    assert_eq!(window.start, 250);
    assert_eq!(ids_of(&window), vec!["250", "251", "252"]);
    assert_eq!(ids_of(&past_the_end), vec!["299"]);
}

#[test]
fn the_counts_say_how_many_tasks_are_in_each_state_in_the_lens_order() {
    let folder = TempDir::new().unwrap();
    let (mut replica, file) = replica_in(&folder).unwrap();
    store_roots(
        &mut replica,
        &[
            root("1", "work", 0, "a", task_in("ready")),
            root("2", "work", 0, "b", task_in("ready")),
            root("3", "work", 0, "c", task_in("blocked")),
            root("4", "work", 0, "d", task_in("completed")),
        ],
    )
    .unwrap();

    let reader = reader_of(&file).unwrap();

    assert_eq!(
        reader.task_counts_in(None),
        vec![
            count("blocked", 1),
            count("failed", 0),
            count("claimed", 0),
            count("ready", 2),
            count("waiting", 0),
            count("completed", 1),
            count("canceled", 0)
        ]
    );
}

#[test]
fn the_counts_are_narrowed_to_a_channel_and_the_channels_under_it() {
    let folder = TempDir::new().unwrap();
    let (mut replica, file) = replica_in(&folder).unwrap();
    store_roots(
        &mut replica,
        &[
            root("1", "fleet", 0, "a", task_in("ready")),
            root("2", "fleet/www", 0, "b", task_in("ready")),
            root("3", "fleetwide", 0, "c", task_in("ready")),
            root("4", "general", 0, "d", task_in("failed")),
        ],
    )
    .unwrap();

    let reader = reader_of(&file).unwrap();
    let counts = reader.task_counts_in(Some("fleet"));

    assert!(counts.contains(&count("ready", 2)));
    assert!(counts.contains(&count("failed", 0)));
}

#[test]
fn a_channel_narrows_the_tasks_to_it_and_the_channels_under_it() {
    let folder = TempDir::new().unwrap();
    let (mut replica, file) = replica_in(&folder).unwrap();
    store_roots(
        &mut replica,
        &[
            root("1", "fleet", 3, "a", task_in("ready")),
            root("2", "fleet/www", 2, "b", task_in("ready")),
            root("3", "fleetwide", 1, "c", task_in("ready")),
        ],
    )
    .unwrap();

    let reader = reader_of(&file).unwrap();
    let narrowed = reader.task_window_in(&in_channel("fleet"), 0, 10).unwrap();

    assert_eq!(narrowed.total, 2);
    assert_eq!(ids_of(&narrowed), vec!["1", "2"]);
    assert_eq!(
        all_ids(&reader, &in_channel("fleet/www")).unwrap(),
        vec!["2"]
    );
}

#[test]
fn choosing_a_state_narrows_the_tasks_to_it() {
    let folder = TempDir::new().unwrap();
    let (mut replica, file) = replica_in(&folder).unwrap();
    store_roots(
        &mut replica,
        &[
            root("1", "work", 0, "a", task_in("ready")),
            root("2", "work", 0, "b", task_in("blocked")),
            root("3", "work", 1, "c", task_in("blocked")),
        ],
    )
    .unwrap();

    let reader = reader_of(&file).unwrap();

    assert_eq!(
        all_ids(&reader, &in_state("blocked")).unwrap(),
        vec!["3", "2"]
    );
    assert_eq!(
        all_ids(&reader, &in_state("canceled")).unwrap(),
        Vec::<String>::new()
    );
}

#[test]
fn the_title_filter_narrows_by_the_first_line_of_the_root_whatever_its_case() {
    let folder = TempDir::new().unwrap();
    let (mut replica, file) = replica_in(&folder).unwrap();
    store_roots(
        &mut replica,
        &[
            root("1", "work", 0, "Fix CVE-123\nin acme/web", task_in("ready")),
            root("2", "work", 0, "deploy acme/api", task_in("ready")),
        ],
    )
    .unwrap();

    let reader = reader_of(&file).unwrap();

    assert_eq!(all_ids(&reader, &titled("cve")).unwrap(), vec!["1"]);
    assert_eq!(all_ids(&reader, &titled("ACME")).unwrap(), vec!["2"]);
    assert_eq!(
        only_row(&reader, "1").unwrap().unwrap().title,
        "Fix CVE-123"
    );
}

#[test]
fn a_channel_a_state_and_a_title_narrow_the_tasks_together() {
    let folder = TempDir::new().unwrap();
    let (mut replica, file) = replica_in(&folder).unwrap();
    store_roots(
        &mut replica,
        &[
            root("1", "fleet/www", 0, "fix the build", task_in("failed")),
            root("2", "fleet/www", 0, "fix the docs", task_in("ready")),
            root("3", "general", 0, "fix the build", task_in("failed")),
            root("4", "fleet/api", 0, "deploy", task_in("failed")),
        ],
    )
    .unwrap();

    let reader = reader_of(&file).unwrap();
    let narrowing = TaskNarrowing {
        channel: Some(String::from("fleet")),
        state: Some(String::from("failed")),
        title: String::from("fix"),
    };

    assert_eq!(all_ids(&reader, &narrowing).unwrap(), vec!["1"]);
}

#[test]
fn a_claimed_task_says_who_owns_it_where_and_who_holds_it_for_how_long() {
    let folder = TempDir::new().unwrap();
    let (mut replica, file) = replica_in(&folder).unwrap();
    let made = root("812", "fleet/www", 0, "fix CVE-123", task_in("ready"));
    let mut claimed = newer(&made, 2);
    claimed["task"]["state"] = Value::from("claimed");
    claimed["task"]["claim"] = json!({ "actor": CLAUDE, "until": time_at(20) });
    store_roots(&mut replica, &[made]).unwrap();
    store_events(
        &mut replica,
        &[event(
            "event-1",
            "task.claimed",
            5,
            (CLAUDE, agent_named("claude")),
            json!({ "until": time_at(20) }),
            &claimed,
        )],
    )
    .unwrap();

    let reader = reader_of(&file).unwrap();

    assert_eq!(
        only_row(&reader, "812").unwrap().unwrap(),
        TaskRow {
            id: String::from("812"),
            state: String::from("claimed"),
            title: String::from("fix CVE-123"),
            channel: String::from("fleet/www"),
            owner: String::from("ada"),
            holder: Some(String::from("claude")),
            lease: Some(Lease {
                since: Some(time_at(5)),
                until: time_at(20),
            }),
            changed_at: time_at(5),
        }
    );
}

#[test]
fn an_owner_or_holder_the_replica_has_no_details_of_reads_as_its_id() {
    let folder = TempDir::new().unwrap();
    let (mut replica, file) = replica_in(&folder).unwrap();
    let mut task = task_in("claimed");
    task["owner"] = Value::from("local:user:grace");
    task["claim"] = json!({ "actor": "local:user:grace+claude", "until": time_at(20) });
    store_roots(&mut replica, &[root("812", "work", 0, "deploy", task)]).unwrap();

    let row = only_row(&reader_of(&file).unwrap(), "812")
        .unwrap()
        .unwrap();

    assert_eq!(row.owner, "local:user:grace");
    assert_eq!(row.holder.as_deref(), Some("local:user:grace+claude"));
    assert_eq!(
        row.lease,
        Some(Lease {
            since: None,
            until: time_at(20)
        })
    );
}

#[test]
fn a_blocked_or_failed_task_names_who_blocked_or_failed_it() {
    let folder = TempDir::new().unwrap();
    let (mut replica, file) = replica_in(&folder).unwrap();
    let blocked = root("blocked", "work", 0, "ask", task_in("ready"));
    let failed = root("failed", "work", 0, "build", task_in("ready"));
    let mut blocked_now = newer(&blocked, 2);
    blocked_now["task"]["state"] = Value::from("blocked");
    blocked_now["task"]["blocked"] = json!({ "kind": null, "reason": "which key?" });
    let mut failed_now = newer(&failed, 2);
    failed_now["task"]["state"] = Value::from("failed");
    failed_now["task"]["reason"] = Value::from("tsc found 3 errors");
    store_roots(&mut replica, &[blocked, failed]).unwrap();
    store_events(
        &mut replica,
        &[
            event(
                "event-1",
                "task.blocked",
                3,
                (CODEX, agent_named("codex")),
                json!({ "reason": "which key?" }),
                &blocked_now,
            ),
            event(
                "event-2",
                "task.failed",
                4,
                (CLAUDE, agent_named("claude")),
                json!({ "reason": "tsc found 3 errors" }),
                &failed_now,
            ),
        ],
    )
    .unwrap();

    let reader = reader_of(&file).unwrap();

    assert_eq!(
        only_row(&reader, "blocked")
            .unwrap()
            .unwrap()
            .holder
            .as_deref(),
        Some("codex")
    );
    assert_eq!(
        only_row(&reader, "failed")
            .unwrap()
            .unwrap()
            .holder
            .as_deref(),
        Some("claude")
    );
    assert_eq!(only_row(&reader, "failed").unwrap().unwrap().lease, None);
}

#[test]
fn a_task_that_changes_moves_to_its_place_once_the_reader_catches_up() {
    let folder = TempDir::new().unwrap();
    let (mut replica, file) = replica_in(&folder).unwrap();
    let quiet = root("quiet", "work", 1, "quiet", task_in("ready"));
    let busy = root("busy", "work", 0, "busy", task_in("ready"));
    store_roots(&mut replica, &[quiet, busy.clone()]).unwrap();
    let mut reader = reader_of(&file).unwrap();
    assert_eq!(
        all_ids(&reader, &in_state("ready")).unwrap(),
        vec!["quiet", "busy"]
    );

    let mut blocked = newer(&busy, 2);
    blocked["task"]["state"] = Value::from("blocked");
    blocked["task"]["blocked"] = json!({ "kind": null, "reason": "which key?" });
    store_events(
        &mut replica,
        &[event(
            "event-1",
            "task.blocked",
            7,
            (CLAUDE, agent_named("claude")),
            json!({ "reason": "which key?" }),
            &blocked,
        )],
    )
    .unwrap();
    let caught_up = reader.catch_up_tasks().unwrap();

    assert!(caught_up);
    assert_eq!(
        all_ids(&reader, &everything()).unwrap(),
        vec!["busy", "quiet"]
    );
    assert_eq!(all_ids(&reader, &in_state("ready")).unwrap(), vec!["quiet"]);
    assert_eq!(
        only_row(&reader, "busy").unwrap().unwrap().changed_at,
        time_at(7)
    );
    assert!(reader.task_counts_in(None).contains(&count("blocked", 1)));
    assert!(!reader.catch_up_tasks().unwrap());
}

#[test]
fn a_task_made_after_the_reader_opened_joins_the_list_once_it_catches_up() {
    let folder = TempDir::new().unwrap();
    let (mut replica, file) = replica_in(&folder).unwrap();
    store_roots(
        &mut replica,
        &[root("old", "work", 0, "old", task_in("ready"))],
    )
    .unwrap();
    let mut reader = reader_of(&file).unwrap();

    store_roots(
        &mut replica,
        &[root("new", "work/www", 5, "new", task_in("ready"))],
    )
    .unwrap();
    reader.catch_up_tasks().unwrap();

    assert_eq!(all_ids(&reader, &everything()).unwrap(), vec!["new", "old"]);
    assert_eq!(
        all_ids(&reader, &in_channel("work/www")).unwrap(),
        vec!["new"]
    );
}

#[test]
fn the_title_follows_an_edit_of_the_root() {
    let folder = TempDir::new().unwrap();
    let (mut replica, file) = replica_in(&folder).unwrap();
    let made = root("812", "work", 0, "fix the build", task_in("ready"));
    store_roots(&mut replica, std::slice::from_ref(&made)).unwrap();
    let mut reader = reader_of(&file).unwrap();

    let mut edited = newer(&made, 2);
    edited["text"] = Value::from("Fix the release build");
    store_roots(&mut replica, &[edited]).unwrap();
    reader.catch_up_tasks().unwrap();

    assert_eq!(
        only_row(&reader, "812").unwrap().unwrap().title,
        "Fix the release build"
    );
    assert_eq!(all_ids(&reader, &titled("release")).unwrap(), vec!["812"]);
}

#[test]
fn a_task_in_a_state_the_lens_does_not_know_comes_last_and_is_counted() {
    let folder = TempDir::new().unwrap();
    let (mut replica, file) = replica_in(&folder).unwrap();
    store_roots(
        &mut replica,
        &[
            root("new", "work", 9, "review", task_in("reviewing")),
            root("done", "work", 0, "build", task_in("canceled")),
        ],
    )
    .unwrap();

    let reader = reader_of(&file).unwrap();

    assert_eq!(
        all_ids(&reader, &everything()).unwrap(),
        vec!["done", "new"]
    );
    assert_eq!(
        reader.task_counts_in(None).last(),
        Some(&count("reviewing", 1))
    );
    assert_eq!(
        all_ids(&reader, &in_state("reviewing")).unwrap(),
        vec!["new"]
    );
}

#[test]
fn a_message_that_is_no_task_is_not_listed() {
    let folder = TempDir::new().unwrap();
    let (mut replica, file) = replica_in(&folder).unwrap();
    store_roots(&mut replica, &[root("1", "work", 0, "hello", Value::Null)]).unwrap();

    let window = reader_of(&file)
        .unwrap()
        .task_window_in(&everything(), 0, 10)
        .unwrap();

    assert_eq!(window.total, 0);
    assert_eq!(window.rows, Vec::new());
}

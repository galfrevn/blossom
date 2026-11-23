#[path = "support/outdated.rs"]
mod outdated;

use blossom::BoardAccess;
use blossom::replica::{
    Actor, Batch, BoardIdentity, Claim, Hold, MessageView, Replica, ReplicaError, Task, TaskEvent,
    Via, known_replica_files, new_replica_file,
};
use outdated::write_a_replica_from_before_task_events;
use serde_json::{Value, json};
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

fn views(views: Vec<MessageView>) -> Batch {
    Batch::from_views(views)
}

fn open_replica_in(folder: &TempDir) -> Result<Replica, ReplicaError> {
    Replica::open(&folder.path().join("board.sqlite"))
}

#[test]
fn a_newer_view_replaces_the_stored_one() {
    let folder = TempDir::new().unwrap();
    let mut replica = open_replica_in(&folder).unwrap();

    replica
        .apply(&views(vec![view("812", 812, "deploy failed").unwrap()]))
        .unwrap();
    replica
        .apply(&views(vec![
            view("812", 815, "deploy failed, rolled back").unwrap(),
        ]))
        .unwrap();

    let stored = replica.stored_view("812").unwrap().unwrap();
    assert_eq!(stored["text"], "deploy failed, rolled back");
}

#[test]
fn an_older_view_never_replaces_a_newer_one_whatever_order_they_arrive_in() {
    let folder = TempDir::new().unwrap();
    let mut replica = open_replica_in(&folder).unwrap();

    replica
        .apply(&views(vec![
            view("812", 815, "deploy failed, rolled back").unwrap(),
        ]))
        .unwrap();
    replica
        .apply(&views(vec![view("812", 812, "deploy failed").unwrap()]))
        .unwrap();

    let stored = replica.stored_view("812").unwrap().unwrap();
    assert_eq!(stored["text"], "deploy failed, rolled back");
}

#[test]
fn the_replica_counts_each_message_once_however_many_views_of_it_arrive() {
    let folder = TempDir::new().unwrap();
    let mut replica = open_replica_in(&folder).unwrap();

    replica
        .apply(&views(vec![
            view("812", 812, "deploy failed").unwrap(),
            view("813", 813, "looking").unwrap(),
            view("812", 815, "deploy failed, rolled back").unwrap(),
        ]))
        .unwrap();

    assert_eq!(replica.message_count(), 2);
}

#[test]
fn what_the_replica_holds_survives_reopening_it() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&views(vec![view("812", 812, "deploy failed").unwrap()]))
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
fn each_new_replica_of_an_address_gets_a_file_of_its_own() {
    let folder = TempDir::new().unwrap();
    let replicas_folder = folder.path().join("replicas");
    let port_4100 = BoardAccess::new("http://127.0.0.1:4100/api/board", "token-a");

    let first = new_replica_file(&replicas_folder, &port_4100).unwrap();
    let second = new_replica_file(&replicas_folder, &port_4100).unwrap();

    assert_ne!(first, second);
    assert_eq!(first.parent(), Some(replicas_folder.as_path()));
    assert!(first.exists());
    assert!(second.exists());
}

#[test]
fn the_known_replicas_of_an_address_are_every_replica_made_for_it() {
    let folder = TempDir::new().unwrap();
    let port_4100 = BoardAccess::new("http://127.0.0.1:4100/api/board", "token-a");
    let port_4200 = BoardAccess::new("http://127.0.0.1:4200/api/board", "token-b");
    let first_at_4100 = new_replica_file(folder.path(), &port_4100).unwrap();
    let second_at_4100 = new_replica_file(folder.path(), &port_4100).unwrap();
    new_replica_file(folder.path(), &port_4200).unwrap();

    let mut known = known_replica_files(folder.path(), &port_4100);
    known.sort();

    let mut expected = vec![first_at_4100, second_at_4100];
    expected.sort();
    assert_eq!(known, expected);
}

#[test]
fn a_replica_is_whole_only_once_its_copy_is_finished() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&views(vec![view("10", 1, "deploy www").unwrap()]))
        .unwrap();
    assert_eq!(replica.feed_cursor().unwrap(), None);

    replica
        .finish_copy(Some(&identity("10", "2026-11-20T10:00:00.000Z")), "815")
        .unwrap();
    drop(replica);
    let reopened = Replica::open(&file).unwrap();

    assert_eq!(
        reopened.identity().unwrap(),
        Some(identity("10", "2026-11-20T10:00:00.000Z"))
    );
    assert_eq!(reopened.feed_cursor().unwrap(), Some(String::from("815")));
}

#[test]
fn a_copy_of_an_empty_board_is_whole_with_no_identity() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();

    replica.finish_copy(None, "0").unwrap();

    assert_eq!(replica.feed_cursor().unwrap(), Some(String::from("0")));
    assert_eq!(replica.identity().unwrap(), None);
}

#[test]
fn the_feed_cursor_is_kept_with_the_views_it_came_with_across_reopening() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    assert_eq!(replica.feed_cursor().unwrap(), None);

    replica
        .apply_from_feed(
            &views(vec![view("812", 812, "deploy failed").unwrap()]),
            "812",
        )
        .unwrap();
    replica
        .apply_from_feed(&views(vec![view("813", 813, "looking").unwrap()]), "815")
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
        .apply(&views(vec![
            view("812", 812, "deploy failed").unwrap(),
            view("813", 813, "looking").unwrap(),
            view("812", 815, "deploy failed, rolled back").unwrap(),
            view("813", 811, "an older look").unwrap(),
            view("812", 815, "deploy failed, rolled back").unwrap(),
        ]))
        .unwrap();
    replica
        .apply_from_feed(&views(vec![view("813", 820, "found it").unwrap()]), "820")
        .unwrap();
    replica
        .apply_from_feed(&views(vec![view("814", 821, "fixed").unwrap()]), "821")
        .unwrap();
    let counted_while_open = replica.message_count();
    drop(replica);

    assert_eq!(counted_while_open, 3);
    assert_eq!(Replica::open(&file).unwrap().message_count(), 3);
}

const CLAUDE: &str = "local:user:julian+claude";
const TRIAGE: &str = "local:user:julian+triage";
const JULIAN: &str = "local:user:julian";

fn claude_named(name: &str) -> Value {
    json!({
        "id": CLAUDE,
        "kind": "agent",
        "handle": "julian/claude",
        "name": name,
        "avatar": null,
        "via": { "name": "julian", "environment": null }
    })
}

fn by_claude() -> Value {
    json!({ "actor": CLAUDE, "principal": JULIAN, "profile": "claude" })
}

fn task_in_state(state: &str) -> Value {
    json!({
        "state": state,
        "owner": JULIAN,
        "by": TRIAGE,
        "for": null,
        "needs": [],
        "epoch": 1,
        "claim": null,
        "blocked": null,
        "result": null,
        "reason": null
    })
}

fn task_root(id: &str, version: u64, task: Value) -> Value {
    json!({
        "id": id,
        "channel": "work",
        "thread": format!("thread-of-{id}"),
        "re": null,
        "createdAt": "2026-11-20T10:00:00.000Z",
        "version": version,
        "text": "fix CVE-123 in acme/web",
        "task": task
    })
}

fn task_root_view(id: &str, version: u64, task: Value) -> Result<MessageView, serde_json::Error> {
    MessageView::parse(&task_root(id, version, task).to_string())
}

fn event_by_claude(
    cursor: &str,
    event_type: &str,
    created_at: &str,
    payload: Value,
    message: Value,
) -> Value {
    json!({
        "cursor": cursor,
        "type": event_type,
        "createdAt": created_at,
        "payload": payload,
        "message": message,
        "actor": claude_named("claude")
    })
}

fn batch_of(events: &[Value]) -> Result<Batch, serde_json::Error> {
    let mut batch = Batch::default();
    for event in events {
        batch.add_event(&event.to_string())?;
    }
    Ok(batch)
}

#[test]
fn a_task_event_is_kept_with_its_cursor_type_task_actor_time_and_fields() {
    let folder = TempDir::new().unwrap();
    let mut replica = open_replica_in(&folder).unwrap();
    let payload = json!({ "id": "812", "thread": "thread-of-812", "epoch": 2, "reason": "the tests fail", "by": by_claude() });
    let failed = event_by_claude(
        "815",
        "task.failed",
        "2026-11-20T10:05:00.000Z",
        payload.clone(),
        task_root("812", 815, task_in_state("failed")),
    );

    replica.apply(&batch_of(&[failed]).unwrap()).unwrap();

    assert_eq!(
        replica.task_events("812").unwrap(),
        vec![TaskEvent {
            cursor: String::from("815"),
            event_type: String::from("task.failed"),
            task: String::from("812"),
            actor: Some(String::from(CLAUDE)),
            created_at: String::from("2026-11-20T10:05:00.000Z"),
            reason: Some(String::from("the tests fail")),
            result: None,
            assigned_to: None,
            needs: None,
            payload,
        }]
    );
}

#[test]
fn a_task_event_keeps_whom_the_task_is_for_what_it_needs_and_what_it_came_to() {
    let folder = TempDir::new().unwrap();
    let mut replica = open_replica_in(&folder).unwrap();
    let created = event_by_claude(
        "813",
        "task.created",
        "2026-11-20T10:01:00.000Z",
        json!({ "id": "812", "thread": "thread-of-812", "needs": ["810", "811"], "for": "local:user:julian+deployer", "by": by_claude() }),
        task_root("812", 813, task_in_state("waiting")),
    );
    let completed = event_by_claude(
        "820",
        "task.completed",
        "2026-11-20T10:09:00.000Z",
        json!({ "id": "812", "thread": "thread-of-812", "epoch": 1, "result": "fixed in a1b2c3", "by": by_claude() }),
        task_root("812", 820, task_in_state("completed")),
    );

    replica
        .apply(&batch_of(&[created, completed]).unwrap())
        .unwrap();

    let events = replica.task_events("812").unwrap();
    assert_eq!(
        events[0].assigned_to.as_deref(),
        Some("local:user:julian+deployer")
    );
    assert_eq!(
        events[0].needs,
        Some(vec![String::from("810"), String::from("811")])
    );
    assert_eq!(events[1].result.as_deref(), Some("fixed in a1b2c3"));
    assert_eq!(events[1].needs, None);
}

#[test]
fn a_task_event_delivered_twice_is_kept_once() {
    let folder = TempDir::new().unwrap();
    let mut replica = open_replica_in(&folder).unwrap();
    let released = event_by_claude(
        "815",
        "task.released",
        "2026-11-20T10:05:00.000Z",
        json!({ "id": "812", "thread": "thread-of-812", "epoch": 1, "by": by_claude() }),
        task_root("812", 815, task_in_state("ready")),
    );

    replica
        .apply(&batch_of(&[released.clone(), released.clone()]).unwrap())
        .unwrap();
    replica.apply(&batch_of(&[released]).unwrap()).unwrap();

    assert_eq!(replica.task_events("812").unwrap().len(), 1);
}

#[test]
fn task_events_are_read_in_the_order_the_feed_delivered_them() {
    let folder = TempDir::new().unwrap();
    let mut replica = open_replica_in(&folder).unwrap();
    let claimed = event_by_claude(
        "9",
        "task.claimed",
        "2026-11-20T10:02:00.000Z",
        json!({ "id": "812", "thread": "thread-of-812", "epoch": 1, "until": "2026-11-20T10:07:00.000Z", "by": by_claude() }),
        task_root("812", 9, task_in_state("claimed")),
    );
    let released = event_by_claude(
        "10",
        "task.released",
        "2026-11-20T10:03:00.000Z",
        json!({ "id": "812", "thread": "thread-of-812", "epoch": 1, "by": by_claude() }),
        task_root("812", 10, task_in_state("ready")),
    );

    replica.apply(&batch_of(&[claimed]).unwrap()).unwrap();
    replica.apply(&batch_of(&[released]).unwrap()).unwrap();

    let cursors: Vec<String> = replica
        .task_events("812")
        .unwrap()
        .into_iter()
        .map(|event| event.cursor)
        .collect();
    assert_eq!(cursors, vec![String::from("9"), String::from("10")]);
}

#[test]
fn only_task_events_are_kept_as_task_events() {
    let folder = TempDir::new().unwrap();
    let mut replica = open_replica_in(&folder).unwrap();
    let posted = event_by_claude(
        "812",
        "message.posted",
        "2026-11-20T10:00:00.000Z",
        json!({ "channel": "work", "text": "fix CVE-123 in acme/web", "re": null, "thread": "thread-of-812", "by": by_claude() }),
        task_root("812", 812, task_in_state("ready")),
    );

    replica.apply(&batch_of(&[posted]).unwrap()).unwrap();

    assert_eq!(replica.task_events("812").unwrap(), Vec::new());
}

#[test]
fn a_task_event_with_no_actor_named_keeps_the_actor_its_payload_names() {
    let folder = TempDir::new().unwrap();
    let mut replica = open_replica_in(&folder).unwrap();
    let unblocked = json!({
        "cursor": "816",
        "type": "task.unblocked",
        "createdAt": "2026-11-20T10:06:00.000Z",
        "payload": { "id": "812", "thread": "thread-of-812", "by": { "actor": TRIAGE, "principal": JULIAN, "profile": "triage" } },
        "message": task_root("812", 816, task_in_state("ready"))
    });

    replica.apply(&batch_of(&[unblocked]).unwrap()).unwrap();

    let events = replica.task_events("812").unwrap();
    assert_eq!(events[0].actor.as_deref(), Some(TRIAGE));
}

#[test]
fn an_expired_lease_is_kept_as_a_task_event_with_no_actor() {
    let folder = TempDir::new().unwrap();
    let mut replica = open_replica_in(&folder).unwrap();
    let expired = json!({
        "cursor": "817",
        "type": "task.expired",
        "createdAt": "2026-11-20T10:12:00.000Z",
        "payload": { "id": "812", "thread": "thread-of-812", "epoch": 1 },
        "message": task_root("812", 817, task_in_state("ready"))
    });

    replica.apply(&batch_of(&[expired]).unwrap()).unwrap();

    let events = replica.task_events("812").unwrap();
    assert_eq!(events[0].event_type, "task.expired");
    assert_eq!(events[0].actor, None);
}

#[test]
fn the_task_index_holds_the_task_its_root_carries() {
    let folder = TempDir::new().unwrap();
    let mut replica = open_replica_in(&folder).unwrap();
    let claimed = json!({
        "state": "claimed",
        "owner": JULIAN,
        "by": TRIAGE,
        "for": CLAUDE,
        "needs": ["810"],
        "epoch": 2,
        "claim": { "actor": CLAUDE, "until": "2026-11-20T10:20:00.000Z" },
        "blocked": null,
        "result": null,
        "reason": null
    });

    replica
        .apply(&views(vec![task_root_view("812", 815, claimed).unwrap()]))
        .unwrap();

    assert_eq!(
        replica.task("812").unwrap(),
        Some(Task {
            id: String::from("812"),
            channel: String::from("work"),
            state: String::from("claimed"),
            owner: String::from(JULIAN),
            made_as: String::from(TRIAGE),
            assigned_to: Some(String::from(CLAUDE)),
            claim: Some(Claim {
                actor: String::from(CLAUDE),
                until: String::from("2026-11-20T10:20:00.000Z"),
            }),
            needs: vec![String::from("810")],
            blocked: None,
            result: None,
            reason: None,
            changed_at: None,
        })
    );
}

#[test]
fn the_task_index_keeps_the_kind_and_reason_of_a_hold() {
    let folder = TempDir::new().unwrap();
    let mut replica = open_replica_in(&folder).unwrap();
    let mut blocked = task_in_state("blocked");
    blocked["blocked"] =
        json!({ "kind": "awaiting_approval", "reason": "approve the prod deploy?" });
    let mut held_without_a_kind = task_in_state("blocked");
    held_without_a_kind["blocked"] = json!({ "kind": null, "reason": "which version?" });

    replica
        .apply(&views(vec![
            task_root_view("812", 815, blocked).unwrap(),
            task_root_view("813", 816, held_without_a_kind).unwrap(),
        ]))
        .unwrap();

    assert_eq!(
        replica.task("812").unwrap().unwrap().blocked,
        Some(Hold {
            kind: Some(String::from("awaiting_approval")),
            reason: String::from("approve the prod deploy?"),
        })
    );
    assert_eq!(
        replica.task("813").unwrap().unwrap().blocked,
        Some(Hold {
            kind: None,
            reason: String::from("which version?"),
        })
    );
}

#[test]
fn the_task_index_keeps_what_a_task_came_to_and_why_one_ended() {
    let folder = TempDir::new().unwrap();
    let mut replica = open_replica_in(&folder).unwrap();
    let mut completed = task_in_state("completed");
    completed["result"] = Value::from("fixed in a1b2c3");
    let mut canceled = task_in_state("canceled");
    canceled["reason"] = Value::from("superseded by the nightly rotation");

    replica
        .apply(&views(vec![
            task_root_view("812", 815, completed).unwrap(),
            task_root_view("813", 816, canceled).unwrap(),
        ]))
        .unwrap();

    let completed = replica.task("812").unwrap().unwrap();
    let canceled = replica.task("813").unwrap().unwrap();
    assert_eq!(completed.result.as_deref(), Some("fixed in a1b2c3"));
    assert_eq!(
        canceled.reason.as_deref(),
        Some("superseded by the nightly rotation")
    );
}

#[test]
fn the_task_index_follows_the_newest_view_of_a_root_whatever_order_views_arrive_in() {
    let folder = TempDir::new().unwrap();
    let mut replica = open_replica_in(&folder).unwrap();

    replica
        .apply(&views(vec![
            task_root_view("812", 815, task_in_state("claimed")).unwrap(),
        ]))
        .unwrap();
    replica
        .apply(&views(vec![
            task_root_view("812", 812, task_in_state("ready")).unwrap(),
        ]))
        .unwrap();
    assert_eq!(replica.task("812").unwrap().unwrap().state, "claimed");

    replica
        .apply(&views(vec![
            task_root_view("812", 820, task_in_state("completed")).unwrap(),
        ]))
        .unwrap();
    assert_eq!(replica.task("812").unwrap().unwrap().state, "completed");
}

#[test]
fn a_message_that_is_not_a_task_has_no_row_in_the_task_index() {
    let folder = TempDir::new().unwrap();
    let mut replica = open_replica_in(&folder).unwrap();

    replica
        .apply(&views(vec![view("812", 812, "deploy failed").unwrap()]))
        .unwrap();

    assert_eq!(replica.task("812").unwrap(), None);
}

#[test]
fn the_task_index_has_the_time_of_the_tasks_last_change_whatever_order_events_arrive_in() {
    let folder = TempDir::new().unwrap();
    let mut replica = open_replica_in(&folder).unwrap();
    let created = event_by_claude(
        "813",
        "task.created",
        "2026-11-20T10:01:00.000Z",
        json!({ "id": "812", "thread": "thread-of-812", "needs": [], "for": null, "by": by_claude() }),
        task_root("812", 813, task_in_state("ready")),
    );
    let claimed = event_by_claude(
        "814",
        "task.claimed",
        "2026-11-20T10:03:00.000Z",
        json!({ "id": "812", "thread": "thread-of-812", "epoch": 1, "until": "2026-11-20T10:08:00.000Z", "by": by_claude() }),
        task_root("812", 814, task_in_state("claimed")),
    );

    replica.apply(&batch_of(&[claimed]).unwrap()).unwrap();
    replica.apply(&batch_of(&[created]).unwrap()).unwrap();

    let task = replica.task("812").unwrap().unwrap();
    assert_eq!(task.state, "claimed");
    assert_eq!(task.changed_at.as_deref(), Some("2026-11-20T10:03:00.000Z"));
}

#[test]
fn the_time_of_a_tasks_last_change_is_kept_when_its_root_arrives_after_it() {
    let folder = TempDir::new().unwrap();
    let mut replica = open_replica_in(&folder).unwrap();
    let claimed_with_no_message = json!({
        "cursor": "814",
        "type": "task.claimed",
        "createdAt": "2026-11-20T10:03:00.000Z",
        "payload": { "id": "812", "thread": "thread-of-812", "epoch": 1, "until": "2026-11-20T10:08:00.000Z", "by": by_claude() },
        "actor": claude_named("claude")
    });

    replica
        .apply(&batch_of(&[claimed_with_no_message]).unwrap())
        .unwrap();
    replica
        .apply(&views(vec![
            task_root_view("812", 814, task_in_state("claimed")).unwrap(),
        ]))
        .unwrap();

    assert_eq!(
        replica.task("812").unwrap().unwrap().changed_at.as_deref(),
        Some("2026-11-20T10:03:00.000Z")
    );
}

fn claimed_by_claude_until(until: &str) -> Value {
    let mut claimed = task_in_state("claimed");
    claimed["claim"] = json!({ "actor": CLAUDE, "until": until });
    claimed
}

fn lease_payload(until: &str) -> Value {
    json!({ "id": "812", "thread": "thread-of-812", "epoch": 1, "until": until, "by": by_claude() })
}

fn claim_then_renewal() -> [Value; 2] {
    let claimed = event_by_claude(
        "814",
        "task.claimed",
        "2026-11-20T10:03:00.000Z",
        lease_payload("2026-11-20T10:08:00.000Z"),
        task_root(
            "812",
            814,
            claimed_by_claude_until("2026-11-20T10:08:00.000Z"),
        ),
    );
    let renewed = event_by_claude(
        "815",
        "task.renewed",
        "2026-11-20T10:06:00.000Z",
        lease_payload("2026-11-20T10:11:00.000Z"),
        task_root(
            "812",
            815,
            claimed_by_claude_until("2026-11-20T10:11:00.000Z"),
        ),
    );
    [claimed, renewed]
}

#[test]
fn a_renewal_moves_the_claim_but_not_the_time_of_the_tasks_last_change() {
    let folder = TempDir::new().unwrap();
    let mut replica = open_replica_in(&folder).unwrap();
    let [claimed, renewed] = claim_then_renewal();

    replica.apply(&batch_of(&[claimed]).unwrap()).unwrap();
    replica.apply(&batch_of(&[renewed]).unwrap()).unwrap();

    let task = replica.task("812").unwrap().unwrap();
    assert_eq!(
        task.claim.map(|claim| claim.until).as_deref(),
        Some("2026-11-20T10:11:00.000Z")
    );
    assert_eq!(task.changed_at.as_deref(), Some("2026-11-20T10:03:00.000Z"));
}

#[test]
fn a_renewal_is_not_the_tasks_last_change_when_its_root_arrives_after_it() {
    let folder = TempDir::new().unwrap();
    let mut replica = open_replica_in(&folder).unwrap();
    let events_with_no_message = claim_then_renewal().map(|mut event| {
        event.as_object_mut().map(|event| event.remove("message"));
        event
    });

    replica
        .apply(&batch_of(&events_with_no_message).unwrap())
        .unwrap();
    replica
        .apply(&views(vec![
            task_root_view(
                "812",
                815,
                claimed_by_claude_until("2026-11-20T10:11:00.000Z"),
            )
            .unwrap(),
        ]))
        .unwrap();

    assert_eq!(
        replica.task("812").unwrap().unwrap().changed_at.as_deref(),
        Some("2026-11-20T10:03:00.000Z")
    );
}

#[test]
fn the_actor_behind_an_event_is_kept() {
    let folder = TempDir::new().unwrap();
    let mut replica = open_replica_in(&folder).unwrap();
    let claimed = event_by_claude(
        "814",
        "task.claimed",
        "2026-11-20T10:03:00.000Z",
        json!({ "id": "812", "thread": "thread-of-812", "epoch": 1, "until": "2026-11-20T10:08:00.000Z", "by": by_claude() }),
        task_root("812", 814, task_in_state("claimed")),
    );

    replica.apply(&batch_of(&[claimed]).unwrap()).unwrap();

    assert_eq!(
        replica.actor(CLAUDE).unwrap(),
        Some(Actor {
            id: String::from(CLAUDE),
            kind: String::from("agent"),
            handle: String::from("julian/claude"),
            name: String::from("claude"),
            via: Some(Via {
                name: String::from("julian"),
                environment: None,
            }),
        })
    );
}

#[test]
fn the_author_of_a_message_view_is_kept_as_an_actor() {
    let folder = TempDir::new().unwrap();
    let mut replica = open_replica_in(&folder).unwrap();
    let posted_by_ada = json!({
        "id": "812",
        "channel": "fleet/www",
        "thread": "thread-of-812",
        "createdAt": "2026-11-20T10:00:00.000Z",
        "version": 812,
        "text": "deploy failed",
        "by": { "actor": "local:user:ada", "principal": "local:user:ada", "profile": null },
        "author": { "kind": "person", "handle": "ada", "name": "Ada Lovelace", "avatar": null }
    });

    replica
        .apply(&views(vec![
            MessageView::parse(&posted_by_ada.to_string()).unwrap(),
        ]))
        .unwrap();

    assert_eq!(
        replica.actor("local:user:ada").unwrap(),
        Some(Actor {
            id: String::from("local:user:ada"),
            kind: String::from("person"),
            handle: String::from("ada"),
            name: String::from("Ada Lovelace"),
            via: None,
        })
    );
}

#[test]
fn the_latest_details_of_an_actor_replace_the_ones_kept() {
    let folder = TempDir::new().unwrap();
    let mut replica = open_replica_in(&folder).unwrap();
    let mut claimed = event_by_claude(
        "814",
        "task.claimed",
        "2026-11-20T10:03:00.000Z",
        json!({ "id": "812", "thread": "thread-of-812", "epoch": 1, "until": "2026-11-20T10:08:00.000Z", "by": by_claude() }),
        task_root("812", 814, task_in_state("claimed")),
    );
    let mut completed = event_by_claude(
        "815",
        "task.completed",
        "2026-11-20T10:04:00.000Z",
        json!({ "id": "812", "thread": "thread-of-812", "epoch": 1, "result": null, "by": by_claude() }),
        task_root("812", 815, task_in_state("completed")),
    );
    claimed["actor"] = claude_named("claude");
    completed["actor"] = claude_named("Claude Opus");

    replica.apply(&batch_of(&[claimed]).unwrap()).unwrap();
    replica.apply(&batch_of(&[completed]).unwrap()).unwrap();

    assert_eq!(replica.actor(CLAUDE).unwrap().unwrap().name, "Claude Opus");
}

#[test]
fn an_actor_never_seen_is_not_known() {
    let folder = TempDir::new().unwrap();
    let replica = open_replica_in(&folder).unwrap();

    assert_eq!(replica.actor("local:user:nobody").unwrap(), None);
}

#[test]
fn a_replica_made_before_task_events_were_kept_is_not_opened() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    write_a_replica_from_before_task_events(&file, "10", "2026-11-20T10:00:00.000Z", "event-3")
        .unwrap();

    let opened = Replica::open(&file);

    assert!(matches!(opened, Err(ReplicaError::AnotherSchemaVersion(0))));
}

struct StoredFacts {
    root: Option<String>,
    author: String,
    plain_agent_reply: bool,
}

fn stored_facts_of(folder: &TempDir, id: &str) -> rusqlite::Result<StoredFacts> {
    let connection = rusqlite::Connection::open(folder.path().join("board.sqlite"))?;
    connection.query_row(
        "SELECT re, author, plain_agent_reply FROM messages WHERE id = ?1",
        [id],
        |row| {
            Ok(StoredFacts {
                root: row.get(0)?,
                author: row.get(1)?,
                plain_agent_reply: row.get(2)?,
            })
        },
    )
}

fn reply_by(
    author: &Value,
    id: &str,
    version: u64,
    extra: &Value,
) -> Result<MessageView, serde_json::Error> {
    let mut board_view = json!({
        "id": id, "channel": "fleet/www", "thread": "thread-of-1", "re": "1",
        "createdAt": "2026-11-20T10:00:00.000Z", "version": version,
        "text": "running the tests", "attachments": [], "author": author
    });
    if let (Some(fields), Some(extra_fields)) = (board_view.as_object_mut(), extra.as_object()) {
        for (key, value) in extra_fields {
            fields.insert(key.clone(), value.clone());
        }
    }
    MessageView::parse(&board_view.to_string())
}

#[test]
fn a_stored_message_keeps_its_root_its_author_and_whether_it_is_a_plain_agent_reply() {
    let folder = TempDir::new().unwrap();
    let mut replica = open_replica_in(&folder).unwrap();
    let agent = json!({ "kind": "agent", "handle": "ada/claude", "name": "claude" });
    let person = json!({ "kind": "person", "handle": "ada", "name": "Ada Lovelace" });
    let attachment = json!({ "id": "a1", "name": "log.txt", "type": "text/plain", "size": 12 });
    replica
        .apply(&views(vec![
            view("1", 1, "fix the flaky test").unwrap(),
            reply_by(&agent, "2", 1, &json!({})).unwrap(),
            reply_by(&person, "3", 1, &json!({})).unwrap(),
            reply_by(
                &agent,
                "4",
                1,
                &json!({ "text": "over to @[local:user:ada]" }),
            )
            .unwrap(),
            reply_by(&agent, "5", 1, &json!({ "attachments": [attachment] })).unwrap(),
            reply_by(
                &Value::Null,
                "6",
                1,
                &json!({ "by": { "actor": "local:user:ada+claude" } }),
            )
            .unwrap(),
            reply_by(
                &agent,
                "7",
                1,
                &json!({ "by": { "actor": "local:user:ada+codex" } }),
            )
            .unwrap(),
        ]))
        .unwrap();

    let root = stored_facts_of(&folder, "1").unwrap();
    assert_eq!(root.root, None);
    assert!(!root.plain_agent_reply);
    let plain = stored_facts_of(&folder, "2").unwrap();
    assert_eq!(plain.root.as_deref(), Some("1"));
    assert_eq!(plain.author, "ada/claude");
    assert!(plain.plain_agent_reply);
    assert!(!stored_facts_of(&folder, "3").unwrap().plain_agent_reply);
    assert!(!stored_facts_of(&folder, "4").unwrap().plain_agent_reply);
    assert!(!stored_facts_of(&folder, "5").unwrap().plain_agent_reply);
    assert_eq!(
        stored_facts_of(&folder, "6").unwrap().author,
        "local:user:ada+claude"
    );
    assert_eq!(
        stored_facts_of(&folder, "7").unwrap().author,
        "local:user:ada+codex"
    );
}

#[test]
fn an_edit_that_adds_a_mention_makes_a_reply_no_longer_plain() {
    let folder = TempDir::new().unwrap();
    let mut replica = open_replica_in(&folder).unwrap();
    let agent = json!({ "kind": "agent", "handle": "ada/claude", "name": "claude" });
    replica
        .apply(&views(vec![reply_by(&agent, "2", 1, &json!({})).unwrap()]))
        .unwrap();

    replica
        .apply(&views(vec![
            reply_by(
                &agent,
                "2",
                2,
                &json!({ "text": "over to @[local:user:ada]" }),
            )
            .unwrap(),
        ]))
        .unwrap();

    assert!(!stored_facts_of(&folder, "2").unwrap().plain_agent_reply);
}

#[test]
fn a_replica_made_before_messages_were_searchable_is_not_opened() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let earlier_replica = rusqlite::Connection::open(&file).unwrap();
    earlier_replica
        .execute_batch(
            "CREATE TABLE messages (id TEXT PRIMARY KEY) WITHOUT ROWID;
             PRAGMA user_version = 1;",
        )
        .unwrap();
    drop(earlier_replica);

    let opened = Replica::open(&file);

    assert!(matches!(opened, Err(ReplicaError::AnotherSchemaVersion(1))));
}

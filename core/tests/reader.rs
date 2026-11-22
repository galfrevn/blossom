use blossom::reader::ReplicaReader;
use blossom::replica::{Batch, MessageView, Replica};
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
        .apply(&Batch::from_views(vec![
            view_in("fleet/www", "1").unwrap(),
            view_in("general", "2").unwrap(),
            view_in("fleet/api", "3").unwrap(),
            view_in("fleet/www", "4").unwrap(),
        ]))
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
    replica
        .apply(&Batch::from_views(vec![view_in("general", "1").unwrap()]))
        .unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    replica
        .apply(&Batch::from_views(vec![
            view_in("fleet/docs", "2").unwrap(),
        ]))
        .unwrap();

    assert_eq!(reader.channels().unwrap(), vec!["fleet/docs", "general"]);
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
        .apply(&Batch::from_views(vec![
            view_at("3", 2, 0, "third").unwrap(),
            view_at("1", 0, 0, "first").unwrap(),
            view_at("2", 1, 0, "second").unwrap(),
        ]))
        .unwrap();

    let reader = ReplicaReader::open(&file).unwrap();
    let window = reader.flow_window_in(None, 0, 10).unwrap();

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
    replica.apply(&Batch::from_views(views)).unwrap();

    let reader = ReplicaReader::open(&file).unwrap();
    let window = reader.flow_window_in(None, 250, 3).unwrap();
    let past_the_end = reader.flow_window_in(None, 299, 10).unwrap();

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
        .apply(&Batch::from_views(vec![
            view_at("1", 0, 0, "first").unwrap(),
            view_at("3", 2, 0, "third").unwrap(),
        ]))
        .unwrap();
    let mut reader = ReplicaReader::open(&file).unwrap();

    replica
        .apply(&Batch::from_views(vec![
            view_at("4", 3, 0, "newest").unwrap(),
            view_at("2", 1, 0, "late").unwrap(),
        ]))
        .unwrap();
    let caught_up = reader.catch_up().unwrap();

    assert!(caught_up);
    assert_eq!(
        ids_of(&reader.flow_window_in(None, 0, 10).unwrap()),
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
        .apply(&Batch::from_views(vec![
            view_at("1", 0, 0, "deploy failed").unwrap(),
        ]))
        .unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    let edited = json!({
        "id": "1", "channel": "fleet/www", "thread": "thread-of-1",
        "createdAt": "2026-11-21T10:00:00.000Z", "version": 2,
        "text": "deploy failed, rolled back", "editedAt": "2026-11-21T10:05:00.000Z"
    });
    replica
        .apply(&Batch::from_views(vec![
            MessageView::parse(&edited.to_string()).unwrap(),
        ]))
        .unwrap();
    let row = reader.flow_window_in(None, 0, 1).unwrap().rows.remove(0);

    assert_eq!(row.text, "deploy failed, rolled back");
    assert!(row.edited);
}

#[test]
fn a_row_says_who_wrote_it_and_for_whom() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&Batch::from_views(vec![
            view_at("1", 0, 0, "on it").unwrap(),
        ]))
        .unwrap();

    let row = ReplicaReader::open(&file)
        .unwrap()
        .flow_window_in(None, 0, 1)
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
        .apply(&Batch::from_views(vec![
            view_at("1", 0, 5, "a").unwrap(),
            view_at("2", 0, 40, "b").unwrap(),
            view_at("3", 1, 2, "c").unwrap(),
        ]))
        .unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    let whole: Vec<bool> = reader
        .flow_window_in(None, 0, 3)
        .unwrap()
        .rows
        .iter()
        .map(|row| row.first_of_its_minute)
        .collect();
    let from_the_middle = reader.flow_window_in(None, 1, 2).unwrap().rows.remove(0);

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
        .apply(&Batch::from_views(vec![
            MessageView::parse(&deleted.to_string()).unwrap(),
        ]))
        .unwrap();

    let row = ReplicaReader::open(&file)
        .unwrap()
        .flow_window_in(None, 0, 1)
        .unwrap()
        .rows
        .remove(0);

    assert!(row.deleted);
}

fn view_in_at(channel: &str, id: &str, minute: u32) -> Result<MessageView, serde_json::Error> {
    let board_view = json!({
        "id": id,
        "channel": channel,
        "thread": format!("thread-of-{id}"),
        "createdAt": format!("2026-11-21T10:{minute:02}:00.000Z"),
        "version": 1,
        "text": "work"
    });
    MessageView::parse(&board_view.to_string())
}

#[test]
fn a_channel_narrows_the_flow_to_itself_and_every_channel_under_it() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&Batch::from_views(vec![
            view_in_at("fleet/www", "1", 0).unwrap(),
            view_in_at("general", "2", 1).unwrap(),
            view_in_at("fleet", "3", 2).unwrap(),
            view_in_at("fleetwide", "4", 3).unwrap(),
            view_in_at("fleet/api/v2", "5", 4).unwrap(),
        ]))
        .unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    let fleet = reader.flow_window_in(Some("fleet"), 0, 10).unwrap();
    let general = reader.flow_window_in(Some("general"), 0, 10).unwrap();

    assert_eq!(ids_of(&fleet), vec!["1", "3", "5"]);
    assert_eq!(fleet.total, 3);
    assert_eq!(ids_of(&general), vec!["2"]);
}

#[test]
fn a_narrowed_flow_reads_windows_and_minutes_within_its_channel() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&Batch::from_views(vec![
            view_in_at("fleet/www", "1", 0).unwrap(),
            view_in_at("general", "2", 1).unwrap(),
            view_in_at("fleet/www", "3", 1).unwrap(),
            view_in_at("fleet/www", "4", 2).unwrap(),
        ]))
        .unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    let window = reader.flow_window_in(Some("fleet/www"), 1, 10).unwrap();

    assert_eq!(window.start, 1);
    assert_eq!(ids_of(&window), vec!["3", "4"]);
    let firsts: Vec<bool> = window
        .rows
        .iter()
        .map(|row| row.first_of_its_minute)
        .collect();
    assert_eq!(firsts, vec![true, true]);
}

#[test]
fn a_narrowed_flow_follows_new_messages_in_its_channel() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&Batch::from_views(vec![
            view_in_at("fleet/www", "1", 0).unwrap(),
        ]))
        .unwrap();
    let mut reader = ReplicaReader::open(&file).unwrap();
    assert_eq!(
        reader.flow_window_in(Some("fleet"), 0, 10).unwrap().total,
        1
    );

    replica
        .apply(&Batch::from_views(vec![
            view_in_at("fleet/docs", "2", 1).unwrap(),
            view_in_at("general", "3", 2).unwrap(),
        ]))
        .unwrap();
    reader.catch_up().unwrap();

    assert_eq!(
        ids_of(&reader.flow_window_in(Some("fleet"), 0, 10).unwrap()),
        vec!["1", "2"]
    );
    assert_eq!(reader.flow_window_in(None, 0, 10).unwrap().total, 3);
}

fn root_task_view(channel: &str, id: &str, second: u32, version: u32) -> serde_json::Value {
    json!({
        "id": id, "channel": channel, "thread": format!("thread-of-{id}"), "re": null,
        "createdAt": format!("2026-11-21T11:00:{second:02}.000Z"), "version": version,
        "text": "fix the flaky test",
        "author": { "kind": "person", "handle": "ada", "name": "Ada Lovelace" },
        "task": {
            "state": "claimed", "owner": "local:user:ada", "by": "local:user:ada", "for": null,
            "needs": [], "epoch": 1, "claim": null, "blocked": null, "result": null, "reason": null
        }
    })
}

fn root_task_in(channel: &str, id: &str, second: u32) -> Result<MessageView, serde_json::Error> {
    MessageView::parse(&root_task_view(channel, id, second, 1).to_string())
}

fn root_made_a_task(
    channel: &str,
    id: &str,
    second: u32,
) -> Result<MessageView, serde_json::Error> {
    MessageView::parse(&root_task_view(channel, id, second, 2).to_string())
}

fn agent_reply_edited(
    id: &str,
    root: &str,
    second: u32,
    text: &str,
) -> Result<MessageView, serde_json::Error> {
    let board_view = json!({
        "id": id, "channel": "fleet/www", "thread": format!("thread-of-{root}"), "re": root,
        "createdAt": format!("2026-11-21T11:00:{second:02}.000Z"), "version": 2,
        "editedAt": "2026-11-21T11:05:00.000Z", "text": text, "attachments": [],
        "author": { "kind": "agent", "handle": "ada/claude", "name": "claude", "via": { "name": "ada", "environment": null } }
    });
    MessageView::parse(&board_view.to_string())
}

struct Reply<'text> {
    id: &'text str,
    root: &'text str,
    channel: &'text str,
    second: u32,
    agent: &'text str,
    text: &'text str,
}

fn reply(reply: &Reply<'_>) -> Result<MessageView, serde_json::Error> {
    let author = if reply.agent.is_empty() {
        json!({ "kind": "person", "handle": "ada", "name": "Ada Lovelace" })
    } else {
        json!({ "kind": "agent", "handle": format!("ada/{}", reply.agent), "name": reply.agent, "via": { "name": "ada", "environment": null } })
    };
    let board_view = json!({
        "id": reply.id, "channel": reply.channel, "thread": format!("thread-of-{}", reply.root),
        "re": reply.root, "createdAt": format!("2026-11-21T11:00:{:02}.000Z", reply.second),
        "version": 1, "text": reply.text, "attachments": [], "author": author
    });
    MessageView::parse(&board_view.to_string())
}

fn root_in(channel: &str, id: &str, second: u32) -> Result<MessageView, serde_json::Error> {
    let board_view = json!({
        "id": id, "channel": channel, "thread": format!("thread-of-{id}"), "re": null,
        "createdAt": format!("2026-11-21T11:00:{second:02}.000Z"), "version": 1, "text": "lunch?",
        "author": { "kind": "person", "handle": "ada", "name": "Ada Lovelace" }
    });
    MessageView::parse(&board_view.to_string())
}

fn progress(
    id: &str,
    root: &str,
    channel: &str,
    second: u32,
    agent: &str,
) -> Result<MessageView, serde_json::Error> {
    reply(&Reply {
        id,
        root,
        channel,
        second,
        agent,
        text: "still running the tests",
    })
}

fn shape_of(window: &blossom::contract::FlowWindow) -> Vec<String> {
    window
        .rows
        .iter()
        .map(|row| match &row.fold {
            Some(fold) => format!("fold {} of {} in {}", fold.count, fold.agents, row.channel),
            None => row.id.clone(),
        })
        .collect()
}

#[test]
fn agents_progress_in_a_tasks_thread_folds_into_one_line_per_channel() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&Batch::from_views(vec![
            root_task_in("fleet/www", "10", 0).unwrap(),
            progress("11", "10", "fleet/www", 1, "claude").unwrap(),
            progress("12", "10", "fleet/www", 2, "codex").unwrap(),
            progress("13", "10", "fleet/www", 3, "claude").unwrap(),
        ]))
        .unwrap();

    let window = ReplicaReader::open(&file)
        .unwrap()
        .flow_window_in(None, 0, 10)
        .unwrap();

    assert_eq!(shape_of(&window), vec!["10", "fold 3 of 2 in fleet/www"]);
    let fold = window.rows[1].fold.as_ref().unwrap();
    let recent: Vec<&str> = fold.recent.iter().map(|row| row.id.as_str()).collect();
    assert_eq!(recent, vec!["11", "12", "13"]);
    assert_eq!(window.rows[1].id, "13");
}

#[test]
fn anything_else_in_the_channel_closes_its_fold() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&Batch::from_views(vec![
            root_task_in("fleet/www", "10", 0).unwrap(),
            progress("11", "10", "fleet/www", 1, "claude").unwrap(),
            progress("12", "10", "fleet/www", 2, "claude").unwrap(),
            reply(&Reply {
                id: "13",
                root: "10",
                channel: "fleet/www",
                second: 3,
                agent: "",
                text: "looks good",
            })
            .unwrap(),
            progress("14", "10", "fleet/www", 4, "claude").unwrap(),
        ]))
        .unwrap();

    let window = ReplicaReader::open(&file)
        .unwrap()
        .flow_window_in(None, 0, 10)
        .unwrap();

    assert_eq!(
        shape_of(&window),
        vec![
            "10",
            "fold 2 of 1 in fleet/www",
            "13",
            "fold 1 of 1 in fleet/www"
        ]
    );
}

#[test]
fn a_fold_sits_where_its_latest_message_sits_and_other_channels_do_not_close_it() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&Batch::from_views(vec![
            root_task_in("fleet/www", "10", 0).unwrap(),
            progress("11", "10", "fleet/www", 1, "claude").unwrap(),
            root_in("general", "12", 30).unwrap(),
            progress("13", "10", "fleet/www", 59, "claude").unwrap(),
        ]))
        .unwrap();

    let window = ReplicaReader::open(&file)
        .unwrap()
        .flow_window_in(None, 0, 10)
        .unwrap();

    assert_eq!(
        shape_of(&window),
        vec!["10", "12", "fold 2 of 1 in fleet/www"]
    );
}

#[test]
fn mentions_files_people_and_threads_that_are_not_tasks_are_never_progress() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    let mut with_a_file = reply(&Reply {
        id: "13",
        root: "10",
        channel: "fleet/www",
        second: 3,
        agent: "claude",
        text: "the log",
    })
    .unwrap();
    let with_a_file_json = json!({
        "id": "13", "channel": "fleet/www", "thread": "thread-of-10", "re": "10",
        "createdAt": "2026-11-21T11:00:03.000Z", "version": 1, "text": "the log",
        "attachments": [{ "id": "a1", "name": "log.txt", "type": "text/plain", "size": 12 }],
        "author": { "kind": "agent", "handle": "ada/claude", "name": "claude", "via": { "name": "ada", "environment": null } }
    });
    with_a_file = MessageView::parse(&with_a_file_json.to_string()).unwrap_or(with_a_file);
    replica
        .apply(&Batch::from_views(vec![
            root_task_in("fleet/www", "10", 0).unwrap(),
            reply(&Reply {
                id: "11",
                root: "10",
                channel: "fleet/www",
                second: 1,
                agent: "claude",
                text: "over to @[local:user:ada]",
            })
            .unwrap(),
            reply(&Reply {
                id: "12",
                root: "10",
                channel: "fleet/www",
                second: 2,
                agent: "",
                text: "thanks",
            })
            .unwrap(),
            with_a_file,
            view_in_at("fleet/api", "20", 0).unwrap(),
            progress("21", "20", "fleet/api", 4, "claude").unwrap(),
        ]))
        .unwrap();

    let window = ReplicaReader::open(&file)
        .unwrap()
        .flow_window_in(None, 0, 10)
        .unwrap();

    assert_eq!(shape_of(&window), vec!["20", "10", "11", "12", "13", "21"]);
}

#[test]
fn a_growing_fold_moves_to_the_bottom() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&Batch::from_views(vec![
            root_task_in("fleet/www", "10", 0).unwrap(),
            progress("11", "10", "fleet/www", 1, "claude").unwrap(),
            root_in("general", "12", 30).unwrap(),
        ]))
        .unwrap();
    let mut reader = ReplicaReader::open(&file).unwrap();
    assert_eq!(
        shape_of(&reader.flow_window_in(None, 0, 10).unwrap()),
        vec!["10", "fold 1 of 1 in fleet/www", "12"]
    );

    replica
        .apply(&Batch::from_views(vec![
            progress("13", "10", "fleet/www", 59, "codex").unwrap(),
        ]))
        .unwrap();
    reader.catch_up().unwrap();

    assert_eq!(
        shape_of(&reader.flow_window_in(None, 0, 10).unwrap()),
        vec!["10", "12", "fold 2 of 2 in fleet/www"]
    );
}

#[test]
fn a_fold_keeps_only_its_last_eight_messages_to_show() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    let mut views = vec![root_task_in("fleet/www", "10", 0).unwrap()];
    for second in 1..=12 {
        let id = format!("1{second:02}");
        views.push(progress(&id, "10", "fleet/www", second, "claude").unwrap());
    }
    replica.apply(&Batch::from_views(views)).unwrap();

    let window = ReplicaReader::open(&file)
        .unwrap()
        .flow_window_in(None, 0, 10)
        .unwrap();

    let fold = window.rows[1].fold.as_ref().unwrap();
    assert_eq!(fold.count, 12);
    let recent: Vec<&str> = fold.recent.iter().map(|row| row.id.as_str()).collect();
    assert_eq!(
        recent,
        vec!["105", "106", "107", "108", "109", "110", "111", "112"]
    );
}

#[test]
fn the_flow_s_order_is_read_from_its_index_alone() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&Batch::from_views(vec![
            root_task_in("fleet/www", "10", 0).unwrap(),
        ]))
        .unwrap();
    let connection = rusqlite::Connection::open(&file).unwrap();
    let mut statement = connection
        .prepare(
            "EXPLAIN QUERY PLAN
             SELECT created_at, id, channel, author,
                    plain_agent_reply AND EXISTS (SELECT 1 FROM tasks WHERE tasks.id = re)
             FROM messages ORDER BY created_at, id",
        )
        .unwrap();
    let plan: Vec<String> = statement
        .query_map([], |row| row.get(3))
        .unwrap()
        .map(Result::unwrap)
        .collect();

    assert!(
        plan.iter()
            .any(|step| step.contains("COVERING INDEX messages_by_time")),
        "{plan:?}"
    );
    assert!(
        !plan.iter().any(|step| step.contains("TEMP B-TREE")),
        "{plan:?}"
    );
}

fn view_numbered(index: u32) -> Result<MessageView, serde_json::Error> {
    view_at(&format!("{index:04}"), index / 60, index % 60, "work")
}

#[test]
fn a_large_batch_of_older_messages_takes_its_places_among_those_already_read() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    let read_first: Vec<MessageView> = (0..1200)
        .step_by(2)
        .map(|index| view_numbered(index).unwrap())
        .collect();
    replica.apply(&Batch::from_views(read_first)).unwrap();
    let mut reader = ReplicaReader::open(&file).unwrap();

    let arriving: Vec<MessageView> = (1..1200)
        .step_by(2)
        .rev()
        .map(|index| view_numbered(index).unwrap())
        .chain([view_numbered(1200).unwrap()])
        .collect();
    replica.apply(&Batch::from_views(arriving)).unwrap();
    reader.catch_up().unwrap();

    let window = reader.flow_window_in(None, 0, 2000).unwrap();
    let expected: Vec<String> = (0..=1200).map(|index| format!("{index:04}")).collect();
    assert_eq!(ids_of(&window), expected);
}

#[test]
fn the_lines_after_a_message_leave_out_those_that_landed_before_it() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&Batch::from_views(vec![
            view_at("2", 2, 0, "second").unwrap(),
            view_at("3", 3, 0, "third").unwrap(),
        ]))
        .unwrap();
    let mut reader = ReplicaReader::open(&file).unwrap();
    let newest = reader.flow_window_in(None, 1, 1).unwrap().rows.remove(0);
    assert_eq!(
        reader.flow_lines_after_in(None, &newest.created_at, &newest.id),
        0
    );

    replica
        .apply(&Batch::from_views(vec![
            view_at("1", 1, 0, "older").unwrap(),
            view_at("0", 0, 0, "oldest").unwrap(),
            view_at("4", 4, 0, "newer").unwrap(),
        ]))
        .unwrap();
    reader.catch_up().unwrap();

    assert_eq!(
        reader.flow_lines_after_in(None, &newest.created_at, &newest.id),
        1
    );
}

#[test]
fn the_lines_after_a_message_in_a_narrowed_flow_are_only_those_of_its_channel() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&Batch::from_views(vec![
            view_in_at("fleet/www", "1", 0).unwrap(),
            view_in_at("fleet/www", "2", 1).unwrap(),
            view_in_at("general", "3", 2).unwrap(),
            view_in_at("fleet/api", "4", 3).unwrap(),
            view_in_at("general", "5", 4).unwrap(),
        ]))
        .unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    let after_the_first =
        reader.flow_lines_after_in(Some("fleet"), "2026-11-21T10:00:00.000Z", "1");

    assert_eq!(after_the_first, 2);
}

#[test]
fn a_fold_that_grew_after_a_message_is_one_line_after_it() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&Batch::from_views(vec![
            root_task_in("fleet/www", "10", 0).unwrap(),
            progress("11", "10", "fleet/www", 1, "claude").unwrap(),
        ]))
        .unwrap();
    let mut reader = ReplicaReader::open(&file).unwrap();
    let fold = reader.flow_window_in(None, 1, 1).unwrap().rows.remove(0);

    replica
        .apply(&Batch::from_views(vec![
            progress("12", "10", "fleet/www", 2, "codex").unwrap(),
            progress("13", "10", "fleet/www", 3, "claude").unwrap(),
        ]))
        .unwrap();
    reader.catch_up().unwrap();

    assert_eq!(
        reader.flow_lines_after_in(None, &fold.created_at, &fold.id),
        1
    );
}

#[test]
fn a_thread_made_into_a_task_folds_the_progress_already_in_it_once_the_reader_catches_up() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&Batch::from_views(vec![
            root_in("fleet/www", "10", 0).unwrap(),
            progress("11", "10", "fleet/www", 1, "claude").unwrap(),
            progress("12", "10", "fleet/www", 2, "codex").unwrap(),
            progress("13", "10", "fleet/www", 3, "claude").unwrap(),
        ]))
        .unwrap();
    let mut reader = ReplicaReader::open(&file).unwrap();
    let before = shape_of(&reader.flow_window_in(None, 0, 10).unwrap());

    replica
        .apply(&Batch::from_views(vec![
            root_made_a_task("fleet/www", "10", 0).unwrap(),
        ]))
        .unwrap();
    let caught_up = reader.catch_up().unwrap();

    assert_eq!(before, vec!["10", "11", "12", "13"]);
    assert!(caught_up);
    assert_eq!(
        shape_of(&reader.flow_window_in(None, 0, 10).unwrap()),
        vec!["10", "fold 3 of 2 in fleet/www"]
    );
    assert!(!reader.catch_up().unwrap());
}

#[test]
fn progress_stored_before_its_task_folds_once_the_task_arrives() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&Batch::from_views(vec![
            progress("11", "10", "fleet/www", 1, "claude").unwrap(),
            progress("12", "10", "fleet/www", 2, "claude").unwrap(),
        ]))
        .unwrap();
    let mut reader = ReplicaReader::open(&file).unwrap();

    replica
        .apply(&Batch::from_views(vec![
            root_task_in("fleet/www", "10", 0).unwrap(),
        ]))
        .unwrap();
    reader.catch_up().unwrap();

    assert_eq!(
        shape_of(&reader.flow_window_in(None, 0, 10).unwrap()),
        vec!["10", "fold 2 of 1 in fleet/www"]
    );
}

#[test]
fn an_edit_that_adds_a_mention_takes_its_reply_out_of_the_fold() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&Batch::from_views(vec![
            root_task_in("fleet/www", "10", 0).unwrap(),
            progress("11", "10", "fleet/www", 1, "claude").unwrap(),
            progress("12", "10", "fleet/www", 2, "claude").unwrap(),
            progress("13", "10", "fleet/www", 3, "claude").unwrap(),
        ]))
        .unwrap();
    let mut reader = ReplicaReader::open(&file).unwrap();

    replica
        .apply(&Batch::from_views(vec![
            agent_reply_edited("12", "10", 2, "over to @[local:user:ada]").unwrap(),
        ]))
        .unwrap();
    let caught_up = reader.catch_up().unwrap();

    assert!(caught_up);
    assert_eq!(
        shape_of(&reader.flow_window_in(None, 0, 10).unwrap()),
        vec![
            "10",
            "fold 1 of 1 in fleet/www",
            "12",
            "fold 1 of 1 in fleet/www"
        ]
    );
}

#[test]
fn an_edit_that_removes_the_mention_brings_its_reply_back_into_the_fold() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&Batch::from_views(vec![
            root_task_in("fleet/www", "10", 0).unwrap(),
            progress("11", "10", "fleet/www", 1, "claude").unwrap(),
            reply(&Reply {
                id: "12",
                root: "10",
                channel: "fleet/www",
                second: 2,
                agent: "claude",
                text: "over to @[local:user:ada]",
            })
            .unwrap(),
            progress("13", "10", "fleet/www", 3, "claude").unwrap(),
        ]))
        .unwrap();
    let mut reader = ReplicaReader::open(&file).unwrap();

    replica
        .apply(&Batch::from_views(vec![
            agent_reply_edited("12", "10", 2, "still running the tests").unwrap(),
        ]))
        .unwrap();
    reader.catch_up().unwrap();

    assert_eq!(
        shape_of(&reader.flow_window_in(None, 0, 10).unwrap()),
        vec!["10", "fold 3 of 1 in fleet/www"]
    );
}

#[test]
fn an_edit_that_changes_none_of_a_message_s_facts_leaves_the_order_as_it_was() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&Batch::from_views(vec![
            root_task_in("fleet/www", "10", 0).unwrap(),
            progress("11", "10", "fleet/www", 1, "claude").unwrap(),
        ]))
        .unwrap();
    let mut reader = ReplicaReader::open(&file).unwrap();

    replica
        .apply(&Batch::from_views(vec![
            agent_reply_edited("11", "10", 1, "still running the tests, 3 left").unwrap(),
        ]))
        .unwrap();
    let caught_up = reader.catch_up().unwrap();
    let window = reader.flow_window_in(None, 0, 10).unwrap();

    assert!(!caught_up);
    assert_eq!(shape_of(&window), vec!["10", "fold 1 of 1 in fleet/www"]);
    let fold = window.rows[1].fold.as_ref().unwrap();
    assert_eq!(fold.recent[0].text, "still running the tests, 3 left");
}

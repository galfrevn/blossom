use std::error::Error;

use blossom::contract::{SearchResults, TextPart};
use blossom::reader::ReplicaReader;
use blossom::replica::{Batch, MessageView, Replica};
use serde_json::{Value, json};
use tempfile::TempDir;

const ADA: &str = "local:user:ada";
const CLAUDE: &str = "local:user:ada+claude";
const CODEX: &str = "local:user:ada+codex";

fn message(id: &str, version: u64, text: &str) -> Value {
    json!({
        "id": id,
        "channel": "fleet/www",
        "thread": format!("thread-of-{id}"),
        "createdAt": "2026-11-21T10:00:00.000Z",
        "version": version,
        "text": text
    })
}

fn message_in(channel: &str, id: &str, text: &str) -> Value {
    let mut view = message(id, 1, text);
    view["channel"] = json!(channel);
    view
}

fn written_by(mut view: Value, actor: &str, kind: &str, handle: &str, name: &str) -> Value {
    view["by"] = json!({ "actor": actor, "principal": ADA, "profile": null });
    view["author"] = json!({ "kind": kind, "handle": handle, "name": name, "avatar": null });
    view
}

fn task_root(id: &str, version: u64, state: &str, text: &str) -> Value {
    let mut view = message(id, version, text);
    view["task"] = json!({
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
    });
    view
}

fn reply_in_thread_of(root: &str, id: &str, text: &str) -> Value {
    let mut view = message(id, 1, text);
    view["thread"] = json!(format!("thread-of-{root}"));
    view
}

fn batch_of(views: &[Value]) -> Result<Batch, serde_json::Error> {
    let parsed = views
        .iter()
        .map(|view| MessageView::parse(&view.to_string()))
        .collect::<Result<Vec<MessageView>, serde_json::Error>>()?;
    Ok(Batch::from_views(parsed))
}

fn message_ids(results: &SearchResults) -> Vec<&str> {
    results
        .messages
        .iter()
        .map(|found| found.id.as_str())
        .collect()
}

fn sorted_message_ids(results: &SearchResults) -> Vec<&str> {
    let mut ids = message_ids(results);
    ids.sort_unstable();
    ids
}

fn marked_words(parts: &[TextPart]) -> Vec<&str> {
    parts
        .iter()
        .filter(|part| part.marked)
        .map(|part| part.text.as_str())
        .collect()
}

fn whole_text(parts: &[TextPart]) -> String {
    parts.iter().map(|part| part.text.as_str()).collect()
}

fn finds_nothing(results: &SearchResults) -> bool {
    results.messages.is_empty()
        && results.tasks.is_empty()
        && results.channels.is_empty()
        && results.agents.is_empty()
}

#[test]
fn a_word_finds_every_message_with_a_word_that_begins_with_it() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(
            &batch_of(&[
                message("1", 1, "deploy failed"),
                message("2", 1, "the deployment is done"),
                message("3", 1, "redeploy tomorrow"),
            ])
            .unwrap(),
        )
        .unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    let results = reader.search("depl").unwrap();

    assert_eq!(sorted_message_ids(&results), vec!["1", "2"]);
}

#[test]
fn every_word_of_a_query_has_to_match() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(
            &batch_of(&[
                message("1", 1, "deploy failed"),
                message("2", 1, "deploy done"),
            ])
            .unwrap(),
        )
        .unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    let results = reader.search("fail depl").unwrap();

    assert_eq!(message_ids(&results), vec!["1"]);
}

#[test]
fn quoted_words_match_only_as_a_phrase() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(
            &batch_of(&[
                message("1", 1, "deploy failed"),
                message("2", 1, "failed to deploy"),
            ])
            .unwrap(),
        )
        .unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    let phrase = reader.search("\"deploy failed\"").unwrap();
    let words = reader.search("deploy failed").unwrap();

    assert_eq!(message_ids(&phrase), vec!["1"]);
    assert_eq!(sorted_message_ids(&words), vec!["1", "2"]);
}

#[test]
fn a_quoted_word_matches_only_as_a_whole_word() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(
            &batch_of(&[
                message("1", 1, "deploy failed"),
                message("2", 1, "the deployment is done"),
            ])
            .unwrap(),
        )
        .unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    let results = reader.search("\"deploy\"").unwrap();

    assert_eq!(message_ids(&results), vec!["1"]);
}

#[test]
fn a_phrase_still_being_typed_matches_the_words_it_begins_with() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(
            &batch_of(&[
                message("1", 1, "deploy failed"),
                message("2", 1, "deploy finished"),
                message("3", 1, "failed to deploy"),
            ])
            .unwrap(),
        )
        .unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    let results = reader.search("\"deploy fai").unwrap();

    assert_eq!(message_ids(&results), vec!["1"]);
}

#[test]
fn a_word_matches_whatever_its_accents() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&batch_of(&[message("1", 1, "the café build is ready")]).unwrap())
        .unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    let results = reader.search("cafe").unwrap();

    assert_eq!(message_ids(&results), vec!["1"]);
}

#[test]
fn an_edit_changes_what_a_message_matches() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&batch_of(&[message("1", 1, "deploy failed")]).unwrap())
        .unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    replica
        .apply(&batch_of(&[message("1", 2, "rolled back")]).unwrap())
        .unwrap();

    assert!(message_ids(&reader.search("deploy").unwrap()).is_empty());
    assert_eq!(message_ids(&reader.search("rolled").unwrap()), vec!["1"]);
}

#[test]
fn an_older_view_that_arrives_late_changes_nothing_that_matches() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&batch_of(&[message("1", 2, "rolled back")]).unwrap())
        .unwrap();

    replica
        .apply(&batch_of(&[message("1", 1, "deploy failed")]).unwrap())
        .unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    assert!(message_ids(&reader.search("deploy").unwrap()).is_empty());
    assert_eq!(message_ids(&reader.search("rolled").unwrap()), vec!["1"]);
}

#[test]
fn a_deleted_message_matches_nothing() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    let mut deleted_later = message("1", 2, "deploy failed");
    deleted_later["deleted"] = json!(true);
    let mut deleted_before_it_arrived = message("2", 2, "deploy failed again");
    deleted_before_it_arrived["deleted"] = json!(true);
    replica
        .apply(&batch_of(&[message("1", 1, "deploy failed")]).unwrap())
        .unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    replica
        .apply(&batch_of(&[deleted_later, deleted_before_it_arrived]).unwrap())
        .unwrap();

    assert!(message_ids(&reader.search("deploy").unwrap()).is_empty());
}

#[test]
fn the_words_that_matched_are_marked() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&batch_of(&[message("1", 1, "the deploy failed on www")]).unwrap())
        .unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    let results = reader.search("depl www").unwrap();

    assert_eq!(
        results.messages[0].snippet,
        vec![
            TextPart {
                text: String::from("the "),
                marked: false
            },
            TextPart {
                text: String::from("deploy"),
                marked: true
            },
            TextPart {
                text: String::from(" failed on "),
                marked: false
            },
            TextPart {
                text: String::from("www"),
                marked: true
            },
        ]
    );
}

#[test]
fn a_long_message_is_cut_to_the_part_around_what_matched() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    let words_before: Vec<String> = (0..40).map(|index| format!("before{index}")).collect();
    let words_after: Vec<String> = (0..40).map(|index| format!("after{index}")).collect();
    let long_text = format!(
        "{} deploy {}",
        words_before.join(" "),
        words_after.join(" ")
    );
    replica
        .apply(&batch_of(&[message("1", 1, &long_text)]).unwrap())
        .unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    let snippet = reader.search("deploy").unwrap().messages.remove(0).snippet;

    let shown = whole_text(&snippet);
    assert!(shown.len() < long_text.len());
    assert!(shown.starts_with('…'));
    assert!(shown.ends_with('…'));
    assert_eq!(marked_words(&snippet), vec!["deploy"]);
}

#[test]
fn the_closest_match_comes_first() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(
            &batch_of(&[
                message(
                    "1",
                    1,
                    "notes from the weekly sync, where the deploy came up among many other things",
                ),
                message("2", 1, "deploy failed"),
            ])
            .unwrap(),
        )
        .unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    let results = reader.search("deploy").unwrap();

    assert_eq!(message_ids(&results), vec!["2", "1"]);
}

#[test]
fn when_more_than_ten_thousand_messages_match_only_the_newest_stored_are_ranked() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    let mut views = vec![message("oldest", 1, "deploy")];
    views.extend((0..10_000).map(|index| {
        message(
            &format!("{index}"),
            1,
            "notes from the weekly sync, where the deploy came up among many other things",
        )
    }));
    replica.apply(&batch_of(&views).unwrap()).unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    let results = reader.search("deploy").unwrap();

    assert_eq!(results.messages.len(), 50);
    assert!(!message_ids(&results).contains(&"oldest"));
}

#[test]
fn a_message_found_says_where_when_and_by_whom() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    let posted = written_by(
        message_in("fleet/api", "1", "deploy failed"),
        CLAUDE,
        "agent",
        "ada/claude",
        "claude",
    );
    replica.apply(&batch_of(&[posted]).unwrap()).unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    let found = reader.search("deploy").unwrap().messages.remove(0);

    assert_eq!(found.id, "1");
    assert_eq!(found.channel, "fleet/api");
    assert_eq!(found.author, "claude");
    assert_eq!(found.created_at, "2026-11-21T10:00:00.000Z");
}

#[test]
fn the_reader_finds_what_the_replica_stores_after_it_opened() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    replica
        .apply(&batch_of(&[message("1", 1, "deploy failed")]).unwrap())
        .unwrap();

    assert_eq!(message_ids(&reader.search("deploy").unwrap()), vec!["1"]);
}

#[test]
fn a_task_is_found_by_the_text_of_its_root_with_its_state() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(
            &batch_of(&[
                task_root("1", 1, "blocked", "fix the login page"),
                reply_in_thread_of("1", "2", "login works on staging"),
                message("3", 1, "login is slow"),
            ])
            .unwrap(),
        )
        .unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    let results = reader.search("login").unwrap();

    assert_eq!(results.tasks.len(), 1);
    assert_eq!(results.tasks[0].id, "1");
    assert_eq!(results.tasks[0].channel, "fleet/www");
    assert_eq!(results.tasks[0].state, "blocked");
    assert_eq!(marked_words(&results.tasks[0].snippet), vec!["login"]);
    assert_eq!(sorted_message_ids(&results), vec!["1", "2", "3"]);
}

#[test]
fn a_task_is_found_as_the_newest_view_of_its_root_stands() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&batch_of(&[task_root("1", 1, "open", "fix the login page")]).unwrap())
        .unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    replica
        .apply(&batch_of(&[task_root("1", 2, "completed", "fix the signup page")]).unwrap())
        .unwrap();

    assert!(reader.search("login").unwrap().tasks.is_empty());
    let found = reader.search("signup").unwrap().tasks;
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].state, "completed");
}

#[test]
fn channels_are_found_by_their_path() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(
            &batch_of(&[
                message_in("fleet/www", "1", "deploy failed"),
                message_in("fleet/api", "2", "deploy failed"),
                message_in("general", "3", "deploy failed"),
            ])
            .unwrap(),
        )
        .unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    let fleet = reader.search("fle").unwrap().channels;
    let www = reader.search("www").unwrap().channels;

    let mut fleet_paths: Vec<&str> = fleet.iter().map(|found| found.path.as_str()).collect();
    fleet_paths.sort_unstable();
    assert_eq!(fleet_paths, vec!["fleet/api", "fleet/www"]);
    assert_eq!(www.len(), 1);
    assert_eq!(www[0].path, "fleet/www");
    assert_eq!(whole_text(&www[0].marked_path), "fleet/www");
    assert_eq!(marked_words(&www[0].marked_path), vec!["www"]);
}

#[test]
fn agents_are_found_by_name_or_handle_and_people_are_not() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(
            &batch_of(&[
                written_by(
                    message("1", 1, "on it"),
                    CLAUDE,
                    "agent",
                    "ada/claude",
                    "claude",
                ),
                written_by(
                    message("2", 1, "on it"),
                    CODEX,
                    "agent",
                    "ada/codex",
                    "codex",
                ),
                written_by(
                    message("3", 1, "thanks"),
                    ADA,
                    "person",
                    "ada",
                    "Ada Lovelace",
                ),
            ])
            .unwrap(),
        )
        .unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    let by_name = reader.search("clau").unwrap().agents;
    let by_handle = reader.search("ada").unwrap().agents;

    assert_eq!(by_name.len(), 1);
    assert_eq!(by_name[0].id, CLAUDE);
    assert_eq!(marked_words(&by_name[0].marked_name), vec!["claude"]);
    assert_eq!(whole_text(&by_name[0].marked_handle), "ada/claude");
    let mut handle_ids: Vec<&str> = by_handle.iter().map(|found| found.id.as_str()).collect();
    handle_ids.sort_unstable();
    assert_eq!(handle_ids, vec![CLAUDE, CODEX]);
    assert_eq!(marked_words(&by_handle[0].marked_handle), vec!["ada"]);
}

#[test]
fn an_agent_is_found_by_its_latest_name_and_handle() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(
            &batch_of(&[written_by(
                message("1", 1, "on it"),
                CLAUDE,
                "agent",
                "ada/triage",
                "triage",
            )])
            .unwrap(),
        )
        .unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    replica
        .apply(
            &batch_of(&[written_by(
                message("2", 1, "on it"),
                CLAUDE,
                "agent",
                "ada/planner",
                "planner",
            )])
            .unwrap(),
        )
        .unwrap();

    assert!(reader.search("triage").unwrap().agents.is_empty());
    assert_eq!(reader.search("plan").unwrap().agents.len(), 1);
}

#[test]
fn each_group_holds_at_most_fifty_results() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    let roots: Vec<Value> = (0..60)
        .map(|index| {
            let mut root = task_root(&format!("{index}"), 1, "open", "deploy the release");
            root["channel"] = json!(format!("deploys/area{index}"));
            written_by(
                root,
                &format!("local:user:ada+deployer{index}"),
                "agent",
                &format!("ada/deployer{index}"),
                &format!("deployer{index}"),
            )
        })
        .collect();
    replica.apply(&batch_of(&roots).unwrap()).unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    let results = reader.search("deploy").unwrap();

    assert_eq!(results.messages.len(), 50);
    assert_eq!(results.tasks.len(), 50);
    assert_eq!(results.channels.len(), 50);
    assert_eq!(results.agents.len(), 50);
}

#[test]
fn a_query_with_no_words_finds_nothing() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(&batch_of(&[message("1", 1, "deploy failed")]).unwrap())
        .unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    for query in ["", "   ", "\"", "\"\"", " \" \" "] {
        assert!(finds_nothing(&reader.search(query).unwrap()), "{query:?}");
    }
}

#[test]
fn the_search_syntax_in_a_query_is_read_as_plain_words() {
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file).unwrap();
    replica
        .apply(
            &batch_of(&[
                message("1", 1, "deploy or roll back"),
                message("2", 1, "deploy failed"),
            ])
            .unwrap(),
        )
        .unwrap();
    let reader = ReplicaReader::open(&file).unwrap();

    assert_eq!(message_ids(&reader.search("deploy OR").unwrap()), vec!["1"]);
    assert!(message_ids(&reader.search("NOT deploy").unwrap()).is_empty());
    for query in [
        "-", "*", "(", ")", "^", ":", "+", "NEAR(", "deploy*", "{text}:", "AND", "\"\"\"", "'",
    ] {
        assert!(reader.search(query).is_ok(), "{query:?}");
    }
}

fn replica_where_claude_wrote(
    folder: &TempDir,
    views: &[Value],
) -> Result<ReplicaReader, Box<dyn Error>> {
    let file = folder.path().join("board.sqlite");
    let mut replica = Replica::open(&file)?;
    let mut written = vec![written_by(
        message("by-claude", 1, "on it"),
        CLAUDE,
        "agent",
        "ada/claude",
        "claude",
    )];
    written.extend_from_slice(views);
    replica.apply(&batch_of(&written)?)?;
    Ok(ReplicaReader::open(&file)?)
}

#[test]
fn a_snippet_reads_a_mention_as_the_name_of_whom_it_mentions_or_as_an_id_it_does_not_know() {
    let folder = TempDir::new().unwrap();
    let reader = replica_where_claude_wrote(
        &folder,
        &[message(
            "1",
            1,
            "deploy failed, ask @[local:user:ada+claude] or @[local:user:nobody]",
        )],
    )
    .unwrap();

    let snippet = reader.search("deploy").unwrap().messages.remove(0).snippet;

    assert_eq!(
        whole_text(&snippet),
        "deploy failed, ask @claude or @local:user:nobody"
    );
    assert_eq!(marked_words(&snippet), vec!["deploy"]);
}

#[test]
fn a_mention_whose_words_matched_is_marked_whole() {
    let folder = TempDir::new().unwrap();
    let reader = replica_where_claude_wrote(
        &folder,
        &[message("1", 1, "over to @[local:user:ada+claude] now")],
    )
    .unwrap();

    let snippet = reader.search("claude").unwrap().messages.remove(0).snippet;

    assert_eq!(whole_text(&snippet), "over to @claude now");
    assert_eq!(marked_words(&snippet), vec!["@claude"]);
}

#[test]
fn a_snippet_cut_inside_a_mention_reads_the_whole_mention() {
    let folder = TempDir::new().unwrap();
    let ends_inside = "one two three four five six seven eight nine ten eleven twelve thirteen \
                       fourteen fifteen sixteen seventeen eighteen nineteen deploy failed, ask \
                       @[local:user:ada+claude] about it twenty twentyone twentytwo twentythree";
    let starts_inside =
        "a b c d e f g h i j @[local:user:ada+claude] x y deploy z1 z2 z3 z4 z5 z6 z7 z8 z9";
    let reader = replica_where_claude_wrote(
        &folder,
        &[
            message("ends-inside", 1, ends_inside),
            message("starts-inside", 1, starts_inside),
        ],
    )
    .unwrap();

    let results = reader.search("deploy").unwrap();
    let snippet_of = |id: &str| {
        let found = results
            .messages
            .iter()
            .find(|found| found.id == id)
            .unwrap();
        whole_text(&found.snippet)
    };

    assert_eq!(
        snippet_of("ends-inside"),
        "…fifteen sixteen seventeen eighteen nineteen deploy failed, ask @claude…"
    );
    assert_eq!(
        snippet_of("starts-inside"),
        "…@claude x y deploy z1 z2 z3 z4 z5 z6…"
    );
}

#[test]
fn a_task_s_snippet_reads_mentions_the_same_way() {
    let folder = TempDir::new().unwrap();
    let reader = replica_where_claude_wrote(
        &folder,
        &[task_root(
            "10",
            1,
            "ready",
            "fix the flaky test, @[local:user:ada+claude]",
        )],
    )
    .unwrap();

    let snippet = reader.search("flaky").unwrap().tasks.remove(0).snippet;

    assert_eq!(whole_text(&snippet), "fix the flaky test, @claude");
}

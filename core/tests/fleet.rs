#[path = "support/answers.rs"]
mod answers;
#[path = "support/history.rs"]
pub mod history;

use std::error::Error;
use std::path::PathBuf;
use std::time::Duration;

use blossom::contract::{Agent, HeldTask, ReplicaState};
use blossom::reader::ReplicaReader;
use blossom::replica::Replica;
use blossom::{BoardAccess, Pacing, copy_board, follow_feed};
use history::{BoardHistory, FakeActor, FakeBoard, lease_until, serve_board, time_at};
use tempfile::TempDir;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};
use wiremock::MockServer;

fn julian() -> FakeActor {
    FakeActor::person("julian", "Julian Benegas")
}

fn deployer() -> FakeActor {
    FakeActor::agent("deployer", "julian")
}

fn reviewer() -> FakeActor {
    FakeActor::agent("reviewer", "julian")
}

fn builder() -> FakeActor {
    FakeActor::agent("builder", "grace")
}

fn quick_pacing() -> Pacing {
    Pacing {
        first_pause: Duration::from_millis(1),
        longest_pause: Duration::from_millis(5),
        failures_before_asking_the_board: 3,
        longest_wait_for_a_stream: Duration::from_millis(200),
        longest_silence: Duration::from_secs(5),
    }
}

fn a_board_with_tasks() -> BoardHistory {
    let mut history = BoardHistory::default();
    history.post("10", "work", "10", 0, "deploy acme/web");
    history.post("11", "work", "11", 1, "review the deploy");
    history.create_task("10", &julian(), 2, None, &[]);
    history.create_task("11", &julian(), 3, None, &[]);
    history
}

struct Copied {
    _server: MockServer,
    _folder: TempDir,
    file: PathBuf,
}

async fn copied(history: &BoardHistory) -> Result<Copied, Box<dyn Error>> {
    let server = MockServer::start().await;
    serve_board(&server, FakeBoard::with_history(history.clone())).await;
    let access = BoardAccess::new(format!("{}/api/board", server.uri()), "token-of-ada");
    let folder = TempDir::new()?;
    let file = folder.path().join("board.sqlite");
    copy_board(&access, Replica::open(&file)?, quick_pacing(), |_| {}).await?;
    Ok(Copied {
        _server: server,
        _folder: folder,
        file,
    })
}

async fn fleet_of(history: &BoardHistory, now: &str) -> Result<Vec<Agent>, Box<dyn Error>> {
    let copy = copied(history).await?;
    let reader = ReplicaReader::open(&copy.file)?;
    Ok(reader.fleet(now)?)
}

fn idle(actor: &FakeActor, on_behalf_of: &str, channel: &str) -> Agent {
    Agent {
        id: actor.id.clone(),
        name: actor.name.clone(),
        on_behalf_of: Some(String::from(on_behalf_of)),
        holds: None,
        lapsed: false,
        failed_recently: false,
        channel: Some(String::from(channel)),
    }
}

fn holding(task: &str, claimed_at_minute: u32) -> Option<HeldTask> {
    Some(HeldTask {
        task: String::from(task),
        until: lease_until(claimed_at_minute),
    })
}

#[tokio::test]
async fn an_agent_holding_a_claim_holds_its_task_until_its_lease_ends() {
    let mut history = a_board_with_tasks();
    history.claim_task("10", &deployer(), 4);

    let fleet = fleet_of(&history, &time_at(6)).await.unwrap();

    assert_eq!(
        fleet,
        vec![Agent {
            holds: holding("10", 4),
            ..idle(&deployer(), "julian", "work")
        }]
    );
}

#[tokio::test]
async fn an_agent_whose_claim_ended_holds_nothing() {
    let mut history = a_board_with_tasks();
    history.claim_task("10", &deployer(), 4);
    history.complete_task("10", &deployer(), 5, "deployed");

    let fleet = fleet_of(&history, &time_at(6)).await.unwrap();

    assert_eq!(fleet, vec![idle(&deployer(), "julian", "work")]);
}

#[tokio::test]
async fn an_agent_whose_latest_claim_expired_has_lapsed() {
    let mut history = a_board_with_tasks();
    history.claim_task("10", &deployer(), 4);
    history.expire_task("10", 9);

    let fleet = fleet_of(&history, &time_at(10)).await.unwrap();

    assert_eq!(
        fleet,
        vec![Agent {
            lapsed: true,
            ..idle(&deployer(), "julian", "work")
        }]
    );
}

#[tokio::test]
async fn an_agent_whose_lease_passed_while_the_board_says_claimed_has_lapsed() {
    let mut history = a_board_with_tasks();
    history.claim_task("10", &deployer(), 4);

    let before_the_lease_ends = fleet_of(&history, &lease_until(4)).await.unwrap();
    let after_the_lease_ends = fleet_of(&history, &time_at(10)).await.unwrap();

    assert!(!before_the_lease_ends[0].lapsed);
    assert_eq!(
        after_the_lease_ends,
        vec![Agent {
            holds: holding("10", 4),
            lapsed: true,
            ..idle(&deployer(), "julian", "work")
        }]
    );
}

#[tokio::test]
async fn an_agent_that_claims_again_after_its_claim_expired_has_not_lapsed() {
    let mut history = a_board_with_tasks();
    history.claim_task("10", &deployer(), 4);
    history.expire_task("10", 9);
    history.claim_task("11", &deployer(), 10);

    let fleet = fleet_of(&history, &time_at(11)).await.unwrap();

    assert_eq!(
        fleet,
        vec![Agent {
            holds: holding("11", 10),
            ..idle(&deployer(), "julian", "work")
        }]
    );
}

#[tokio::test]
async fn another_agent_claiming_an_expired_task_lapses_only_the_one_whose_claim_expired() {
    let mut history = a_board_with_tasks();
    history.claim_task("10", &deployer(), 4);
    history.expire_task("10", 9);
    history.claim_task("10", &reviewer(), 10);

    let fleet = fleet_of(&history, &time_at(11)).await.unwrap();

    assert_eq!(
        fleet,
        vec![
            Agent {
                lapsed: true,
                ..idle(&deployer(), "julian", "work")
            },
            Agent {
                holds: holding("10", 10),
                ..idle(&reviewer(), "julian", "work")
            },
        ]
    );
}

#[tokio::test]
async fn an_agent_whose_task_failed_in_the_last_hour_failed_recently() {
    let mut history = a_board_with_tasks();
    history.claim_task("10", &deployer(), 4);
    history.fail_task("10", &deployer(), 5, "the tests fail");

    let within_the_hour = fleet_of(&history, &time_at(64)).await.unwrap();
    let past_the_hour = fleet_of(&history, &time_at(66)).await.unwrap();

    assert_eq!(
        within_the_hour,
        vec![Agent {
            failed_recently: true,
            ..idle(&deployer(), "julian", "work")
        }]
    );
    assert_eq!(past_the_hour, vec![idle(&deployer(), "julian", "work")]);
}

#[tokio::test]
async fn an_agent_holding_two_claims_holds_the_one_whose_lease_ends_last() {
    let mut history = a_board_with_tasks();
    history.claim_task("10", &deployer(), 4);
    history.claim_task("11", &deployer(), 6);

    let fleet = fleet_of(&history, &time_at(7)).await.unwrap();

    assert_eq!(fleet[0].holds, holding("11", 6));
}

#[tokio::test]
async fn people_are_left_out_of_the_fleet() {
    let history = a_board_with_tasks();

    let fleet = fleet_of(&history, &time_at(6)).await.unwrap();

    assert_eq!(fleet, Vec::new());
}

#[tokio::test]
async fn every_agent_the_replica_has_seen_is_in_the_fleet_by_handle() {
    let mut history = a_board_with_tasks();
    history.know_actor(FakeActor::person("grace", "Grace Hopper"));
    history.claim_task("10", &reviewer(), 4);
    history.post_as(&builder(), "12", "docs", "12", 5, "building the docs");
    history.claim_task("11", &deployer(), 6);

    let fleet = fleet_of(&history, &time_at(7)).await.unwrap();

    assert_eq!(
        fleet,
        vec![
            idle(&builder(), "grace", "docs"),
            Agent {
                holds: holding("11", 6),
                ..idle(&deployer(), "julian", "work")
            },
            Agent {
                holds: holding("10", 4),
                ..idle(&reviewer(), "julian", "work")
            },
        ]
    );
}

#[tokio::test]
async fn an_agents_channel_is_the_channel_of_its_latest_message_or_claim() {
    let mut history = a_board_with_tasks();
    history.claim_task("10", &deployer(), 4);
    history.post_as(&deployer(), "12", "docs", "12", 5, "notes on the deploy");
    let history_until_its_message = history.clone();
    history.complete_task("10", &deployer(), 6, "deployed");
    history.claim_task("11", &deployer(), 7);

    let after_its_message = fleet_of(&history_until_its_message, &time_at(6))
        .await
        .unwrap();
    let after_its_claim = fleet_of(&history, &time_at(8)).await.unwrap();

    assert_eq!(after_its_message[0].channel.as_deref(), Some("docs"));
    assert_eq!(after_its_claim[0].channel.as_deref(), Some("work"));
}

async fn until_live_with(states: &mut UnboundedReceiver<ReplicaState>, stored: u32) -> bool {
    let waiting = async {
        while let Some(state) = states.recv().await {
            if state == (ReplicaState::Live { stored }) {
                return true;
            }
        }
        false
    };
    tokio::time::timeout(Duration::from_secs(5), waiting)
        .await
        .unwrap_or(false)
}

#[tokio::test]
async fn the_fleet_follows_the_replica() {
    let board = FakeBoard::with_history(a_board_with_tasks());
    let server = MockServer::start().await;
    serve_board(&server, board.clone()).await;
    let access = BoardAccess::new(format!("{}/api/board", server.uri()), "token-of-ada");
    let folder = TempDir::new().unwrap();
    let file = folder.path().join("board.sqlite");
    let copy = copy_board(
        &access,
        Replica::open(&file).unwrap(),
        quick_pacing(),
        |_| {},
    )
    .await
    .unwrap();
    let reader = ReplicaReader::open(&file).unwrap();
    assert_eq!(reader.fleet(&time_at(4)).unwrap(), Vec::new());
    let (sender, mut states) = unbounded_channel();
    let following = tokio::spawn(async move {
        follow_feed(
            &access,
            copy.replica,
            copy.feed_cursor,
            quick_pacing(),
            |state| {
                sender.send(state).ok();
            },
        )
        .await
    });

    board.change_history(|history| {
        history.claim_task("10", &deployer(), 4);
        history.post("12", "work", "10", 5, "deploying");
    });

    assert!(until_live_with(&mut states, 3).await);
    following.abort();
    assert_eq!(
        reader.fleet(&time_at(5)).unwrap(),
        vec![Agent {
            holds: holding("10", 4),
            ..idle(&deployer(), "julian", "work")
        }]
    );
}

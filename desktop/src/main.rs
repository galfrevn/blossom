use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use blossom::contract::{OpenBoard, ReplicaState};
use blossom::replica::{Replica, replica_file};
use blossom::{BoardAccess, OpenBoardFollower, copy_board};
use tauri::async_runtime::JoinHandle;
use tauri::ipc::{Channel, IpcResponse};
use tauri::webview::PageLoadEvent;
use tauri::{Manager, State};
use tokio::sync::watch;

const TIME_BETWEEN_LOOKS: Duration = Duration::from_secs(1);

#[derive(Default)]
struct RunningTask {
    task: Mutex<Option<JoinHandle<()>>>,
}

impl RunningTask {
    fn replace(&self, task: JoinHandle<()>) {
        let mut current_task = self.task.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(previous_task) = current_task.replace(task) {
            previous_task.abort();
        }
    }

    fn stop(&self) {
        let mut current_task = self.task.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(previous_task) = current_task.take() {
            previous_task.abort();
        }
    }
}

struct Session {
    replicas_folder: Option<PathBuf>,
    replica_state: watch::Sender<Option<ReplicaState>>,
    following_open_board: RunningTask,
    forwarding_replica_state: RunningTask,
    copying: RunningTask,
}

impl Session {
    fn new(replicas_folder: Option<PathBuf>) -> Session {
        let (replica_state, _) = watch::channel(None);
        Session {
            replicas_folder,
            replica_state,
            following_open_board: RunningTask::default(),
            forwarding_replica_state: RunningTask::default(),
            copying: RunningTask::default(),
        }
    }

    fn start_copying(self: &Arc<Session>, access: BoardAccess) {
        let session = Arc::clone(self);
        let copy = tauri::async_runtime::spawn(async move { session.copy(access).await });
        self.copying.replace(copy);
    }

    fn stop_copying(&self) {
        self.copying.stop();
        self.replica_state.send_replace(None);
    }

    async fn copy(&self, access: BoardAccess) {
        let Some(replicas_folder) = self.replicas_folder.clone() else {
            eprintln!("blossom: there is no data folder to keep the replica in");
            return;
        };
        let file = replica_file(&replicas_folder, &access);
        let opened = tauri::async_runtime::spawn_blocking(move || {
            fs::create_dir_all(&replicas_folder).map_err(|error| error.to_string())?;
            Replica::open(&file).map_err(|error| error.to_string())
        })
        .await;
        let replica = match opened {
            Ok(Ok(replica)) => replica,
            Ok(Err(error)) => {
                eprintln!("blossom: the replica could not be opened: {error}");
                return;
            }
            Err(error) => {
                eprintln!("blossom: the replica could not be opened: {error}");
                return;
            }
        };
        let copied = copy_board(&access, replica, |state| {
            self.replica_state.send_replace(Some(state));
        })
        .await;
        if let Err(error) = copied {
            eprintln!("blossom: the board could not be copied: {error}");
        }
    }
}

#[tauri::command]
fn subscribe_to_open_board(channel: Channel<OpenBoard>, session: State<'_, Arc<Session>>) {
    let following = tauri::async_runtime::spawn(follow_open_board(channel, Arc::clone(&session)));
    session.following_open_board.replace(following);
}

#[tauri::command]
fn subscribe_to_replica(channel: Channel<ReplicaState>, session: State<'_, Arc<Session>>) {
    let receiver = session.replica_state.subscribe();
    let forwarding = tauri::async_runtime::spawn(forward_replica_state(channel, receiver));
    session.forwarding_replica_state.replace(forwarding);
}

async fn follow_open_board(channel: Channel<OpenBoard>, session: Arc<Session>) {
    let Some(token_folder) = blossom::token_folder_from_environment() else {
        send(&channel, OpenBoard::NoneRunning);
        return;
    };
    let mut follower = OpenBoardFollower::new(token_folder);
    loop {
        if let Some(open_board) = follower.look().await {
            match follower.access_to_open_board() {
                Some(access) => session.start_copying(access.clone()),
                None => session.stop_copying(),
            }
            if !send(&channel, open_board) {
                return;
            }
        }
        tokio::time::sleep(TIME_BETWEEN_LOOKS).await;
    }
}

async fn forward_replica_state(
    channel: Channel<ReplicaState>,
    mut receiver: watch::Receiver<Option<ReplicaState>>,
) {
    loop {
        let current_state = receiver.borrow_and_update().clone();
        if let Some(state) = current_state
            && !send(&channel, state)
        {
            return;
        }
        if receiver.changed().await.is_err() {
            return;
        }
    }
}

fn send<Value: IpcResponse>(channel: &Channel<Value>, value: Value) -> bool {
    match channel.send(value) {
        Ok(()) => true,
        Err(error) => {
            eprintln!("blossom: a subscription could not reach the window: {error}");
            false
        }
    }
}

fn main() -> ExitCode {
    let application = tauri::Builder::default()
        .setup(|application| {
            let replicas_folder = application
                .path()
                .app_data_dir()
                .ok()
                .map(|data_folder| data_folder.join("replicas"));
            application.manage(Arc::new(Session::new(replicas_folder)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            subscribe_to_open_board,
            subscribe_to_replica
        ])
        .on_page_load(|webview, payload| {
            if payload.event() != PageLoadEvent::Finished {
                return;
            }
            if let Err(error) = webview.window().show() {
                eprintln!("blossom: the window could not be shown: {error}");
            }
        })
        .run(tauri::generate_context!());
    match application {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("blossom: {error}");
            ExitCode::FAILURE
        }
    }
}

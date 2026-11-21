mod rate;

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use blossom::contract::{OpenBoard, ReplicaState};
use blossom::reader::ReplicaReader;
use blossom::{BoardAccess, KeepingEnded, OpenBoardFollower, Pacing, keep_in_step};
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
    reader: Mutex<Option<ReplicaReader>>,
    channels: watch::Sender<Option<Vec<String>>>,
    following_open_board: RunningTask,
    forwarding_replica_state: RunningTask,
    forwarding_channels: RunningTask,
    keeping_in_step: RunningTask,
    open_board_gone: AtomicBool,
}

impl Session {
    fn new(replicas_folder: Option<PathBuf>) -> Session {
        let (replica_state, _) = watch::channel(None);
        let (channels, _) = watch::channel(None);
        Session {
            replicas_folder,
            replica_state,
            reader: Mutex::new(None),
            channels,
            following_open_board: RunningTask::default(),
            forwarding_replica_state: RunningTask::default(),
            forwarding_channels: RunningTask::default(),
            keeping_in_step: RunningTask::default(),
            open_board_gone: AtomicBool::new(false),
        }
    }

    fn start_keeping_in_step(self: &Arc<Session>, access: BoardAccess) {
        self.forget_the_replica();
        let session = Arc::clone(self);
        let keeping =
            tauri::async_runtime::spawn(async move { session.keep_in_step(access).await });
        self.keeping_in_step.replace(keeping);
    }

    fn stop_keeping_in_step(&self) {
        self.keeping_in_step.stop();
        self.replica_state.send_replace(None);
        self.forget_the_replica();
    }

    fn read_the_replica(&self, file: &Path) {
        match ReplicaReader::open(file) {
            Ok(reader) => {
                *self.reader.lock().unwrap_or_else(PoisonError::into_inner) = Some(reader);
                self.refresh_channels();
            }
            Err(error) => eprintln!("blossom: the replica could not be read: {error}"),
        }
    }

    fn forget_the_replica(&self) {
        *self.reader.lock().unwrap_or_else(PoisonError::into_inner) = None;
        self.channels.send_replace(None);
    }

    fn refresh_channels(&self) {
        let reader = self.reader.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(reader) = reader.as_ref() else {
            return;
        };
        match reader.channels() {
            Ok(channels) => {
                self.channels.send_if_modified(|current| {
                    if current.as_ref() == Some(&channels) {
                        return false;
                    }
                    *current = Some(channels);
                    true
                });
            }
            Err(error) => eprintln!("blossom: the replica's channels could not be read: {error}"),
        }
    }

    async fn keep_in_step(&self, access: BoardAccess) {
        let Some(replicas_folder) = self.replicas_folder.as_deref() else {
            eprintln!("blossom: there is no data folder to keep the replica in");
            return;
        };
        let ended = keep_in_step(
            &access,
            replicas_folder,
            Pacing::for_the_app(),
            |file| self.read_the_replica(file),
            |state| {
                self.replica_state.send_replace(Some(state));
                self.refresh_channels();
            },
        )
        .await;
        match ended {
            KeepingEnded::BoardGone => self.open_board_gone.store(true, Ordering::SeqCst),
            KeepingEnded::Stopped(error) => {
                eprintln!("blossom: the replica stopped: {error}");
            }
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
    let forwarding = tauri::async_runtime::spawn(forward(channel, receiver));
    session.forwarding_replica_state.replace(forwarding);
}

#[tauri::command]
fn subscribe_to_channels(channel: Channel<Vec<String>>, session: State<'_, Arc<Session>>) {
    let receiver = session.channels.subscribe();
    let forwarding = tauri::async_runtime::spawn(forward(channel, receiver));
    session.forwarding_channels.replace(forwarding);
}

async fn follow_open_board(channel: Channel<OpenBoard>, session: Arc<Session>) {
    let Some(token_folder) = blossom::token_folder_from_environment() else {
        send(&channel, OpenBoard::NoneRunning);
        return;
    };
    let mut follower = OpenBoardFollower::new(token_folder);
    loop {
        if session.open_board_gone.swap(false, Ordering::SeqCst) {
            follower.forget_the_open_board();
        }
        if let Some(open_board) = follower.look().await {
            match follower.access_to_open_board() {
                Some(access) => session.start_keeping_in_step(access.clone()),
                None => session.stop_keeping_in_step(),
            }
            if !send(&channel, open_board) {
                return;
            }
        }
        tokio::time::sleep(TIME_BETWEEN_LOOKS).await;
    }
}

async fn forward<Value: IpcResponse + Clone>(
    channel: Channel<Value>,
    mut receiver: watch::Receiver<Option<Value>>,
) {
    loop {
        let current_value = receiver.borrow_and_update().clone();
        if let Some(value) = current_value
            && !send(&channel, value)
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
            if let Some(window) = application.get_webview_window("main")
                && let Err(error) = rate::draw_at_the_display_rate(&window)
            {
                eprintln!("blossom: the window could not draw at the display's rate: {error}");
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            subscribe_to_open_board,
            subscribe_to_replica,
            subscribe_to_channels
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

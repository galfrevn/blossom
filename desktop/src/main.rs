mod link;
mod navigation;
mod rate;
mod tasks;

#[cfg(test)]
mod tests {
    mod link;
    mod navigation;
}

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, SystemTime};

use blossom::contract::{
    Agent, FlowChange, FlowWindow, OpenBoard, ReplicaState, SearchResults, TaskChange, Thread,
};
use blossom::reader::ReplicaReader;
use blossom::time::board_time;
use blossom::{BoardAccess, KeepingEnded, OpenBoardFollower, Pacing, keep_in_step};
use tauri::async_runtime::JoinHandle;
use tauri::ipc::{Channel, IpcResponse};
use tauri::webview::PageLoadEvent;
use tauri::{Manager, State};
use tokio::sync::{Notify, watch};

const TIME_BETWEEN_LOOKS: Duration = Duration::from_secs(1);
const TIME_BETWEEN_FLEET_READS: Duration = Duration::from_secs(1);

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
    replica_changed: Notify,
    channels: watch::Sender<Option<Vec<String>>>,
    flow: watch::Sender<Option<FlowChange>>,
    flow_revision: AtomicU32,
    fleet: watch::Sender<Option<Vec<Agent>>>,
    tasks: watch::Sender<Option<TaskChange>>,
    task_revision: AtomicU32,
    following_open_board: RunningTask,
    forwarding_replica_state: RunningTask,
    forwarding_channels: RunningTask,
    forwarding_flow: RunningTask,
    forwarding_fleet: RunningTask,
    forwarding_tasks: RunningTask,
    keeping_in_step: RunningTask,
    refreshing_from_the_replica: RunningTask,
    rereading_the_fleet: RunningTask,
    open_board_gone: AtomicBool,
}

impl Session {
    fn new(replicas_folder: Option<PathBuf>) -> Session {
        let (replica_state, _) = watch::channel(None);
        let (channels, _) = watch::channel(None);
        let (flow, _) = watch::channel(None);
        let (fleet, _) = watch::channel(None);
        let (tasks, _) = watch::channel(None);
        Session {
            replicas_folder,
            replica_state,
            reader: Mutex::new(None),
            replica_changed: Notify::new(),
            channels,
            flow,
            flow_revision: AtomicU32::new(0),
            fleet,
            tasks,
            task_revision: AtomicU32::new(0),
            following_open_board: RunningTask::default(),
            forwarding_replica_state: RunningTask::default(),
            forwarding_channels: RunningTask::default(),
            forwarding_flow: RunningTask::default(),
            forwarding_fleet: RunningTask::default(),
            forwarding_tasks: RunningTask::default(),
            keeping_in_step: RunningTask::default(),
            refreshing_from_the_replica: RunningTask::default(),
            rereading_the_fleet: RunningTask::default(),
            open_board_gone: AtomicBool::new(false),
        }
    }

    fn start_keeping_in_step(self: &Arc<Session>, access: BoardAccess) {
        self.forget_the_replica();
        let session = Arc::clone(self);
        let refreshing = tauri::async_runtime::spawn(refresh_as_the_replica_changes(session));
        self.refreshing_from_the_replica.replace(refreshing);
        let session = Arc::clone(self);
        let keeping =
            tauri::async_runtime::spawn(async move { session.keep_in_step(access).await });
        self.keeping_in_step.replace(keeping);
        let session = Arc::clone(self);
        let rereading = tauri::async_runtime::spawn(reread_the_fleet_as_time_passes(session));
        self.rereading_the_fleet.replace(rereading);
    }

    fn stop_keeping_in_step(&self) {
        self.keeping_in_step.stop();
        self.refreshing_from_the_replica.stop();
        self.rereading_the_fleet.stop();
        self.replica_state.send_replace(None);
        self.forget_the_replica();
    }

    fn read_the_replica(&self, file: &Path) {
        match ReplicaReader::open(file) {
            Ok(reader) => {
                *self.reader.lock().unwrap_or_else(PoisonError::into_inner) = Some(reader);
                self.refresh_from_the_replica();
            }
            Err(error) => eprintln!("blossom: the replica could not be read: {error}"),
        }
    }

    fn forget_the_replica(&self) {
        *self.reader.lock().unwrap_or_else(PoisonError::into_inner) = None;
        self.channels.send_replace(None);
        self.flow.send_replace(None);
        self.fleet.send_replace(None);
        self.tasks.send_replace(None);
    }

    fn refresh_from_the_replica(&self) {
        let mut reader = self.reader.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(reader) = reader.as_mut() else {
            return;
        };
        if let Err(error) = reader.catch_up() {
            eprintln!("blossom: the replica's new messages could not be read: {error}");
        }
        let revision = self
            .flow_revision
            .fetch_add(1, Ordering::SeqCst)
            .wrapping_add(1);
        self.flow.send_replace(Some(FlowChange { revision }));
        self.catch_up_with_the_tasks(reader);
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
        self.send_the_fleet(reader);
    }

    fn reread_the_fleet(&self) {
        let reader = self.reader.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(reader) = reader.as_ref() else {
            return;
        };
        self.send_the_fleet(reader);
    }

    fn send_the_fleet(&self, reader: &ReplicaReader) {
        let now = board_time(SystemTime::now());
        match reader.fleet(&now) {
            Ok(agents) => {
                self.fleet.send_if_modified(|current| {
                    if current.as_ref() == Some(&agents) {
                        return false;
                    }
                    *current = Some(agents);
                    true
                });
            }
            Err(error) => eprintln!("blossom: the replica's fleet could not be read: {error}"),
        }
    }

    fn read_flow(
        &self,
        channel: Option<&str>,
        start: u32,
        count: u32,
    ) -> Result<FlowWindow, String> {
        let reader = self.reader.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(reader) = reader.as_ref() else {
            return Err(String::from("no replica is open"));
        };
        reader
            .flow_window_in(channel, start, count)
            .map_err(|error| error.to_string())
    }

    fn count_flow_after(
        &self,
        channel: Option<&str>,
        created_at: &str,
        id: &str,
    ) -> Result<u32, String> {
        let reader = self.reader.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(reader) = reader.as_ref() else {
            return Err(String::from("no replica is open"));
        };
        Ok(reader.flow_lines_after_in(channel, created_at, id))
    }

    fn read_thread(&self, message_id: &str) -> Result<Option<Thread>, String> {
        let reader = self.reader.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(reader) = reader.as_ref() else {
            return Err(String::from("no replica is open"));
        };
        reader
            .thread_of(message_id)
            .map_err(|error| error.to_string())
    }

    fn search(&self, query: &str) -> Result<SearchResults, String> {
        let reader = self.reader.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(reader) = reader.as_ref() else {
            return Err(String::from("no replica is open"));
        };
        reader.search(query).map_err(|error| error.to_string())
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
                self.replica_changed.notify_one();
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
fn subscribe_to_flow(channel: Channel<FlowChange>, session: State<'_, Arc<Session>>) {
    let receiver = session.flow.subscribe();
    let forwarding = tauri::async_runtime::spawn(forward(channel, receiver));
    session.forwarding_flow.replace(forwarding);
}

#[tauri::command]
fn read_flow(
    channel: Option<String>,
    start: u32,
    count: u32,
    session: State<'_, Arc<Session>>,
) -> Result<FlowWindow, String> {
    session.read_flow(channel.as_deref(), start, count)
}

#[tauri::command]
fn count_flow_after(
    channel: Option<String>,
    created_at: String,
    id: String,
    session: State<'_, Arc<Session>>,
) -> Result<u32, String> {
    session.count_flow_after(channel.as_deref(), &created_at, &id)
}

#[tauri::command(async)]
fn search(query: String, session: State<'_, Arc<Session>>) -> Result<SearchResults, String> {
    session.search(&query)
}

#[tauri::command]
fn read_thread(
    message_id: String,
    session: State<'_, Arc<Session>>,
) -> Result<Option<Thread>, String> {
    session.read_thread(&message_id)
}

#[tauri::command]
async fn open_link(link: String) -> Result<(), String> {
    let Some(url) = link::link_to_open(&link) else {
        return Err(format!("{link} is not a web or mail link"));
    };
    let opening =
        tauri::async_runtime::spawn_blocking(move || link::open_in_the_default_browser(&url));
    match opening.await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(error)) => Err(format!("{link} could not be opened: {error}")),
        Err(error) => Err(format!("{link} could not be opened: {error}")),
    }
}

#[tauri::command]
fn subscribe_to_channels(channel: Channel<Vec<String>>, session: State<'_, Arc<Session>>) {
    let receiver = session.channels.subscribe();
    let forwarding = tauri::async_runtime::spawn(forward(channel, receiver));
    session.forwarding_channels.replace(forwarding);
}

#[tauri::command]
fn subscribe_to_fleet(channel: Channel<Vec<Agent>>, session: State<'_, Arc<Session>>) {
    let receiver = session.fleet.subscribe();
    let forwarding = tauri::async_runtime::spawn(forward(channel, receiver));
    session.forwarding_fleet.replace(forwarding);
}

async fn refresh_as_the_replica_changes(session: Arc<Session>) {
    loop {
        session.replica_changed.notified().await;
        session.refresh_from_the_replica();
    }
}

async fn reread_the_fleet_as_time_passes(session: Arc<Session>) {
    loop {
        tokio::time::sleep(TIME_BETWEEN_FLEET_READS).await;
        session.reread_the_fleet();
    }
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
        .plugin(navigation::keeping_the_window_on_its_page())
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
            subscribe_to_channels,
            subscribe_to_flow,
            subscribe_to_fleet,
            read_flow,
            count_flow_after,
            read_thread,
            search,
            open_link,
            tasks::subscribe_to_tasks,
            tasks::read_tasks,
            tasks::read_task_counts
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

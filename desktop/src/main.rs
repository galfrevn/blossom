use std::process::ExitCode;
use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use blossom::OpenBoardFollower;
use blossom::contract::OpenBoard;
use tauri::State;
use tauri::async_runtime::JoinHandle;
use tauri::ipc::Channel;
use tauri::webview::PageLoadEvent;

const TIME_BETWEEN_LOOKS: Duration = Duration::from_secs(1);

#[derive(Default)]
struct OpenBoardSubscription {
    following_task: Mutex<Option<JoinHandle<()>>>,
}

#[tauri::command]
fn subscribe_to_open_board(
    channel: Channel<OpenBoard>,
    subscription: State<'_, OpenBoardSubscription>,
) {
    let following_task = tauri::async_runtime::spawn(follow_open_board(channel));
    let mut current_task = subscription
        .following_task
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    if let Some(previous_task) = current_task.replace(following_task) {
        previous_task.abort();
    }
}

async fn follow_open_board(channel: Channel<OpenBoard>) {
    let Some(token_folder) = blossom::token_folder_from_environment() else {
        send_open_board(&channel, OpenBoard::NoneRunning);
        return;
    };
    let mut follower = OpenBoardFollower::new(token_folder);
    loop {
        if let Some(open_board) = follower.look().await
            && !send_open_board(&channel, open_board)
        {
            return;
        }
        tokio::time::sleep(TIME_BETWEEN_LOOKS).await;
    }
}

fn send_open_board(channel: &Channel<OpenBoard>, open_board: OpenBoard) -> bool {
    match channel.send(open_board) {
        Ok(()) => true,
        Err(error) => {
            eprintln!("blossom: the open board could not reach the window: {error}");
            false
        }
    }
}

fn main() -> ExitCode {
    let application = tauri::Builder::default()
        .manage(OpenBoardSubscription::default())
        .invoke_handler(tauri::generate_handler![subscribe_to_open_board])
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

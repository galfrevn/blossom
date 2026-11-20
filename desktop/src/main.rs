use std::process::ExitCode;

use blossom::contract::OpenBoard;
use tauri::ipc::Channel;
use tauri::webview::PageLoadEvent;

#[tauri::command]
async fn subscribe_to_open_board(channel: Channel<OpenBoard>) {
    let open_board = match blossom::token_folder_from_environment() {
        Some(token_folder) => blossom::find_open_board(&token_folder).await,
        None => OpenBoard::NoneRunning,
    };
    if let Err(error) = channel.send(open_board) {
        eprintln!("blossom: the open board could not reach the window: {error}");
    }
}

fn main() -> ExitCode {
    let application = tauri::Builder::default()
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

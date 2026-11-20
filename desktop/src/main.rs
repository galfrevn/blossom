use std::process::ExitCode;

use tauri::webview::PageLoadEvent;

fn main() -> ExitCode {
    let application = tauri::Builder::default()
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

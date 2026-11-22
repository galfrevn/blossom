use tauri::plugin::{Builder, TauriPlugin};
use tauri::{Manager, Runtime, Url};

const SCHEME_OF_THE_PAGE: &str = "tauri";

pub fn stays_on_the_page(destination: &Url, dev_server: Option<&Url>) -> bool {
    if destination.scheme() == SCHEME_OF_THE_PAGE {
        return true;
    }
    dev_server.is_some_and(|dev_server| dev_server.origin() == destination.origin())
}

pub fn keeping_the_window_on_its_page<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("navigation")
        .on_navigation(|webview, destination| {
            let dev_server = if tauri::is_dev() {
                webview.config().build.dev_url.as_ref()
            } else {
                None
            };
            let stays = stays_on_the_page(destination, dev_server);
            if !stays {
                eprintln!(
                    "blossom: the window stayed on its page instead of going to {destination}"
                );
            }
            stays
        })
        .build()
}

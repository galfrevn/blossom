use tauri::Url;

use crate::navigation::stays_on_the_page;

fn url(text: &str) -> Url {
    Url::parse(text).unwrap()
}

#[test]
fn the_window_stays_on_its_own_page() {
    assert!(stays_on_the_page(&url("tauri://localhost/"), None));
    assert!(stays_on_the_page(
        &url("tauri://localhost/index.html"),
        None
    ));
}

#[test]
fn while_developing_the_window_stays_on_the_dev_server() {
    let dev_server = url("http://localhost:5173");

    assert!(stays_on_the_page(
        &url("http://localhost:5173/"),
        Some(&dev_server)
    ));
    assert!(stays_on_the_page(
        &url("http://localhost:5173/src/main.tsx"),
        Some(&dev_server)
    ));
}

#[test]
fn the_window_never_leaves_its_page() {
    for destination in [
        "https://vercel.com/",
        "http://localhost:5173/",
        "http://localhost:4500/api/board",
        "mailto:ada@example.com",
        "file:///etc/hosts",
        "about:blank",
    ] {
        assert!(
            !stays_on_the_page(&url(destination), None),
            "{destination} should be refused"
        );
    }
}

#[test]
fn while_developing_the_window_leaves_for_nowhere_else() {
    let dev_server = url("http://localhost:5173");

    for destination in [
        "https://vercel.com/",
        "http://localhost:5174/",
        "https://localhost:5173/",
    ] {
        assert!(
            !stays_on_the_page(&url(destination), Some(&dev_server)),
            "{destination} should be refused while developing"
        );
    }
}

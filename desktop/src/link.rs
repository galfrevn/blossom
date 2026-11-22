use std::io;
use std::process::Command;

use tauri::Url;

const SCHEMES_THAT_OPEN_IN_THE_BROWSER: [&str; 3] = ["http", "https", "mailto"];
const OPENER_OF_MACOS: &str = "/usr/bin/open";

pub fn link_to_open(link: &str) -> Option<Url> {
    let url = Url::parse(link).ok()?;
    if !SCHEMES_THAT_OPEN_IN_THE_BROWSER.contains(&url.scheme()) {
        return None;
    }
    Some(url)
}

pub fn open_in_the_default_browser(url: &Url) -> io::Result<()> {
    let status = Command::new(OPENER_OF_MACOS).arg(url.as_str()).status()?;
    if status.success() {
        return Ok(());
    }
    Err(io::Error::other(format!(
        "{OPENER_OF_MACOS} ended with {status}"
    )))
}

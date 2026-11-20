use std::cmp::Reverse;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use reqwest::Url;
use serde::Deserialize;

const LOOPBACK_HOSTS: [&str; 3] = ["127.0.0.1", "localhost", "[::1]"];

#[derive(Deserialize)]
pub struct LocalBoard {
    pub url: String,
    pub token: String,
}

pub fn token_folder(xdg_config_home: Option<&OsStr>, home: Option<&OsStr>) -> Option<PathBuf> {
    let configuration_folder = match xdg_config_home {
        Some(xdg_config_home) if !xdg_config_home.is_empty() => PathBuf::from(xdg_config_home),
        _ => Path::new(home?).join(".config"),
    };
    Some(configuration_folder.join("board").join("local"))
}

pub fn token_folder_from_environment() -> Option<PathBuf> {
    let xdg_config_home = env::var_os("XDG_CONFIG_HOME");
    let home = env::var_os("HOME");
    token_folder(xdg_config_home.as_deref(), home.as_deref())
}

pub fn local_boards_newest_first(token_folder: &Path) -> Vec<LocalBoard> {
    let Ok(entries) = fs::read_dir(token_folder) else {
        return Vec::new();
    };
    let mut boards_with_their_write_times: Vec<(SystemTime, LocalBoard)> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| read_local_board(&entry.path()))
        .collect();
    boards_with_their_write_times.sort_by_key(|(written_at, _)| Reverse(*written_at));
    boards_with_their_write_times
        .into_iter()
        .map(|(_, local_board)| local_board)
        .collect()
}

fn read_local_board(token_file: &Path) -> Option<(SystemTime, LocalBoard)> {
    let written_at = fs::metadata(token_file).ok()?.modified().ok()?;
    let contents = fs::read_to_string(token_file).ok()?;
    let local_board: LocalBoard = serde_json::from_str(&contents).ok()?;
    if !is_on_loopback(&local_board.url) {
        return None;
    }
    Some((written_at, local_board))
}

fn is_on_loopback(url: &str) -> bool {
    let Ok(parsed_url) = Url::parse(url) else {
        return false;
    };
    parsed_url
        .host_str()
        .is_some_and(|host| LOOPBACK_HOSTS.contains(&host))
}

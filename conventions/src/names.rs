use std::ffi::OsStr;
use std::path::{Path, PathBuf};

const NAMES_A_TOOL_DICTATES: [&str; 8] = [
    "Cargo.toml",
    "Cargo.lock",
    "README.md",
    "LICENSE",
    "CLAUDE.md",
    "GLOSSARY.md",
    "Tauri.toml",
    "pre-push",
];

pub fn paths_not_named_with_one_word(path: &Path) -> Vec<PathBuf> {
    let mut prefix = PathBuf::new();
    let mut misnamed_paths = Vec::new();
    for component in path.components() {
        prefix.push(component);
        if !is_one_word(component.as_os_str()) {
            misnamed_paths.push(prefix.clone());
        }
    }
    misnamed_paths
}

fn is_one_word(segment: &OsStr) -> bool {
    let Some(segment) = segment.to_str() else {
        return false;
    };
    if NAMES_A_TOOL_DICTATES.contains(&segment) {
        return true;
    }
    if let Some(dotfile_name) = segment.strip_prefix('.') {
        return is_lowercase_word(dotfile_name);
    }
    let Some((stem, extension)) = segment.split_once('.') else {
        return is_lowercase_word(segment);
    };
    is_lowercase_word(stem) && is_lowercase_extension(extension)
}

fn is_lowercase_word(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|character| character.is_ascii_lowercase())
}

fn is_lowercase_extension(extension: &str) -> bool {
    !extension.is_empty()
        && extension
            .chars()
            .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
}

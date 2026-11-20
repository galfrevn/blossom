use std::ffi::OsStr;
use std::path::PathBuf;

use blossom::token_folder;

#[test]
fn the_token_folder_is_under_xdg_config_home_when_it_is_set() {
    let folder = token_folder(
        Some(OsStr::new("/Users/ada/configuration")),
        Some(OsStr::new("/Users/ada")),
    );

    assert_eq!(
        folder,
        Some(PathBuf::from("/Users/ada/configuration/board/local"))
    );
}

#[test]
fn the_token_folder_is_under_dot_config_in_home_otherwise() {
    let folder = token_folder(None, Some(OsStr::new("/Users/ada")));

    assert_eq!(
        folder,
        Some(PathBuf::from("/Users/ada/.config/board/local"))
    );
}

#[test]
fn an_empty_xdg_config_home_counts_as_unset() {
    let folder = token_folder(Some(OsStr::new("")), Some(OsStr::new("/Users/ada")));

    assert_eq!(
        folder,
        Some(PathBuf::from("/Users/ada/.config/board/local"))
    );
}

#[test]
fn there_is_no_token_folder_without_either() {
    assert_eq!(token_folder(None, None), None);
}

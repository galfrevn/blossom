use std::path::PathBuf;

use crate::contract::OpenBoard;
use crate::find_open_board;
use crate::local::{TokenFolderSnapshot, snapshot_token_folder};

pub struct OpenBoardFollower {
    token_folder: PathBuf,
    last_snapshot: Option<TokenFolderSnapshot>,
    last_reported_open_board: Option<OpenBoard>,
}

impl OpenBoardFollower {
    pub fn new(token_folder: PathBuf) -> OpenBoardFollower {
        OpenBoardFollower {
            token_folder,
            last_snapshot: None,
            last_reported_open_board: None,
        }
    }

    pub async fn look(&mut self) -> Option<OpenBoard> {
        let snapshot = snapshot_token_folder(&self.token_folder);
        if self.last_snapshot.as_ref() == Some(&snapshot) {
            return None;
        }
        self.last_snapshot = Some(snapshot);
        let open_board = find_open_board(&self.token_folder).await;
        if self.last_reported_open_board.as_ref() == Some(&open_board) {
            return None;
        }
        self.last_reported_open_board = Some(open_board.clone());
        Some(open_board)
    }
}

use std::path::PathBuf;

use crate::access::BoardAccess;
use crate::contract::OpenBoard;
use crate::find_board;
use crate::local::{TokenFolderSnapshot, snapshot_token_folder};

pub struct OpenBoardFollower {
    token_folder: PathBuf,
    settled_snapshot: Option<TokenFolderSnapshot>,
    last_reported_open_board: Option<OpenBoard>,
    access_to_open_board: Option<BoardAccess>,
}

impl OpenBoardFollower {
    pub fn new(token_folder: PathBuf) -> OpenBoardFollower {
        OpenBoardFollower {
            token_folder,
            settled_snapshot: None,
            last_reported_open_board: None,
            access_to_open_board: None,
        }
    }

    pub async fn look(&mut self) -> Option<OpenBoard> {
        let snapshot = snapshot_token_folder(&self.token_folder);
        if self.settled_snapshot.as_ref() == Some(&snapshot) {
            return None;
        }
        let found_board = find_board(&self.token_folder).await;
        let open_board = found_board
            .as_ref()
            .map_or(OpenBoard::NoneRunning, |found_board| {
                found_board.open_board()
            });
        let nothing_left_to_try = found_board.is_some() || snapshot.has_no_files();
        self.settled_snapshot = nothing_left_to_try.then_some(snapshot);
        self.access_to_open_board = found_board.map(|found_board| found_board.access);
        if self.last_reported_open_board.as_ref() == Some(&open_board) {
            return None;
        }
        self.last_reported_open_board = Some(open_board.clone());
        Some(open_board)
    }

    pub fn access_to_open_board(&self) -> Option<&BoardAccess> {
        self.access_to_open_board.as_ref()
    }
}

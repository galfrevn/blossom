#![forbid(unsafe_code)]

mod access;
pub mod contract;
mod copy;
mod feed;
mod follow;
mod keeping;
mod local;
mod pacing;
mod protocol;
pub mod reader;
pub mod replica;
pub mod sse;
mod storage;

use std::path::Path;

use contract::{OpenBoard, Person};

pub use access::BoardAccess;
pub use copy::{CopiedBoard, CopyError, copy_board};
pub use feed::{FollowError, follow_feed};
pub use follow::OpenBoardFollower;
pub use keeping::{KeepingEnded, keep_in_step};
pub use local::{token_folder, token_folder_from_environment};
pub use pacing::Pacing;
pub use storage::StorageError;

pub async fn find_open_board(token_folder: &Path) -> OpenBoard {
    match find_board(token_folder).await {
        Some(found_board) => found_board.open_board(),
        None => OpenBoard::NoneRunning,
    }
}

pub(crate) struct FoundBoard {
    pub(crate) access: BoardAccess,
    pub(crate) person: Person,
}

impl FoundBoard {
    pub(crate) fn open_board(&self) -> OpenBoard {
        OpenBoard::Open {
            url: self.access.url().to_owned(),
            person: self.person.clone(),
        }
    }
}

pub(crate) async fn find_board(token_folder: &Path) -> Option<FoundBoard> {
    let client = protocol::client()?;
    for access in local::local_boards_newest_first(token_folder) {
        if !protocol::discovery_names_its_own_url(&client, &access).await {
            continue;
        }
        if let Some(person) = protocol::person_on(&client, &access).await {
            return Some(FoundBoard { access, person });
        }
    }
    None
}

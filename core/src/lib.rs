pub mod contract;
mod follow;
mod local;
mod protocol;

use std::path::Path;

use contract::OpenBoard;

pub use follow::OpenBoardFollower;
pub use local::{token_folder, token_folder_from_environment};

pub async fn find_open_board(token_folder: &Path) -> OpenBoard {
    let Some(client) = protocol::client() else {
        return OpenBoard::NoneRunning;
    };
    for local_board in local::local_boards_newest_first(token_folder) {
        if !protocol::discovery_names_its_own_url(&client, &local_board).await {
            continue;
        }
        if let Some(person) = protocol::person_on(&client, &local_board).await {
            return OpenBoard::Open {
                url: local_board.url,
                person,
            };
        }
    }
    OpenBoard::NoneRunning
}

use std::fs::{self, File};
use std::io;
use std::path::Path;
use std::time::{Duration, SystemTime};

use serde_json::json;
use wiremock::MockServer;

mod answers;

pub use answers::{answer_discovery_naming, answer_me};

pub struct FakeBoard {
    pub server: MockServer,
    pub url: String,
}

pub async fn start_fake_board(token: &str, handle: &str, name: &str) -> FakeBoard {
    let server = MockServer::builder().start().await;
    let url = format!("{}/api/board", server.uri());
    answer_discovery_naming(&server, &url).await;
    answer_me(&server, token, handle, name).await;
    FakeBoard { server, url }
}

pub fn write_token_file(
    folder: &Path,
    file_name: &str,
    url: &str,
    token: &str,
    written_seconds_ago: u64,
) -> io::Result<()> {
    let contents = json!({ "url": url, "token": token, "handle": "ada" }).to_string();
    let file_path = folder.join(file_name);
    fs::write(&file_path, contents)?;
    let written_at = SystemTime::now() - Duration::from_secs(written_seconds_ago);
    File::options()
        .write(true)
        .open(&file_path)?
        .set_modified(written_at)
}

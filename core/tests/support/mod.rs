use std::fs::{self, File};
use std::io;
use std::path::Path;
use std::time::{Duration, SystemTime};

use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

pub struct FakeBoard {
    pub server: MockServer,
    pub url: String,
}

pub async fn start_fake_board(token: &str, handle: &str, name: &str) -> FakeBoard {
    let server = MockServer::start().await;
    let url = format!("{}/api/board", server.uri());
    answer_discovery_naming(&server, &url).await;
    answer_me(&server, token, handle, name).await;
    FakeBoard { server, url }
}

pub async fn start_board_whose_discovery_names(
    other_url: &str,
    token: &str,
    handle: &str,
    name: &str,
) -> FakeBoard {
    let server = MockServer::start().await;
    let url = format!("{}/api/board", server.uri());
    answer_discovery_naming(&server, other_url).await;
    answer_me(&server, token, handle, name).await;
    FakeBoard { server, url }
}

async fn answer_me(server: &MockServer, token: &str, handle: &str, name: &str) {
    let me = json!({
        "actor": format!("local:user:{handle}"),
        "principal": format!("local:user:{handle}"),
        "kind": "user",
        "handle": handle,
        "name": name,
        "avatar": null,
        "profile": null,
        "expiresAt": null,
        "can": { "*": ["read", "post"] }
    });
    Mock::given(method("GET"))
        .and(path("/api/board/me"))
        .and(header("authorization", format!("Bearer {token}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(me))
        .mount(server)
        .await;
}

async fn answer_discovery_naming(server: &MockServer, url: &str) {
    let discovery = json!({
        "version": "0.54.0",
        "url": url,
        "audience": url,
        "team_id": null,
        "project_id": null,
        "sync": []
    });
    Mock::given(method("GET"))
        .and(path("/api/board"))
        .respond_with(ResponseTemplate::new(200).set_body_json(discovery))
        .mount(server)
        .await;
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

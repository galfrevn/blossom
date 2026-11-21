use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

pub async fn answer_me(server: &MockServer, token: &str, handle: &str, name: &str) {
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

pub async fn answer_discovery_naming(server: &MockServer, url: &str) {
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

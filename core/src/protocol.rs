use std::time::Duration;

use reqwest::Client;
use reqwest::redirect::Policy;
use serde::Deserialize;

use crate::contract::Person;
use crate::local::LocalBoard;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Deserialize)]
struct Discovery {
    url: String,
}

#[derive(Deserialize)]
struct Me {
    handle: String,
    name: String,
}

pub fn client() -> Option<Client> {
    Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .no_proxy()
        .redirect(Policy::none())
        .build()
        .ok()
}

pub async fn discovery_names_its_own_url(client: &Client, local_board: &LocalBoard) -> bool {
    let Ok(response) = client.get(&local_board.url).send().await else {
        return false;
    };
    let Ok(accepted_response) = response.error_for_status() else {
        return false;
    };
    let Ok(discovery) = accepted_response.json::<Discovery>().await else {
        return false;
    };
    without_trailing_slash(&discovery.url) == without_trailing_slash(&local_board.url)
}

fn without_trailing_slash(url: &str) -> &str {
    url.trim_end_matches('/')
}

pub async fn person_on(client: &Client, local_board: &LocalBoard) -> Option<Person> {
    let me_url = format!("{}/me", local_board.url);
    let response = client
        .get(me_url)
        .bearer_auth(&local_board.token)
        .send()
        .await
        .ok()?;
    let accepted_response = response.error_for_status().ok()?;
    let me: Me = accepted_response.json().await.ok()?;
    Some(Person {
        handle: me.handle,
        name: me.name,
    })
}

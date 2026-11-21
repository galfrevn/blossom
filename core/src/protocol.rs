use std::time::Duration;

use reqwest::Client;
use reqwest::redirect::Policy;
use serde::Deserialize;
use serde_json::value::RawValue;

use crate::access::BoardAccess;
use crate::contract::Person;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const SILENCE_THAT_ENDS_A_STREAM: Duration = Duration::from_secs(45);

#[derive(Deserialize)]
pub(crate) struct FeedEvent {
    pub(crate) message: Option<Box<RawValue>>,
}

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

pub fn streaming_client() -> Option<Client> {
    Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .read_timeout(SILENCE_THAT_ENDS_A_STREAM)
        .no_proxy()
        .redirect(Policy::none())
        .build()
        .ok()
}

pub async fn discovery_names_its_own_url(client: &Client, access: &BoardAccess) -> bool {
    let Ok(response) = client.get(access.url()).send().await else {
        return false;
    };
    let Ok(accepted_response) = response.error_for_status() else {
        return false;
    };
    let Ok(discovery) = accepted_response.json::<Discovery>().await else {
        return false;
    };
    without_trailing_slash(&discovery.url) == without_trailing_slash(access.url())
}

fn without_trailing_slash(url: &str) -> &str {
    url.trim_end_matches('/')
}

pub async fn person_on(client: &Client, access: &BoardAccess) -> Option<Person> {
    let me_url = format!("{}/me", access.url());
    let response = client
        .get(me_url)
        .bearer_auth(access.token())
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

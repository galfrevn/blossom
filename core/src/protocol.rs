use std::time::Duration;

use reqwest::redirect::Policy;
use reqwest::{Client, Url};
use serde::Deserialize;

use crate::access::BoardAccess;
use crate::contract::Person;
use crate::replica::Actor;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
pub(crate) const ACTORS_PER_REQUEST: usize = 100;
pub(crate) const WITH_BOOKKEEPING: (&str, &str) = ("all", "true");

#[derive(Deserialize)]
pub(crate) struct ListedActors {
    pub(crate) actors: Vec<Actor>,
}

pub(crate) fn actors_query(asked_ids: &[String]) -> Vec<(&'static str, &str)> {
    asked_ids.iter().map(|id| ("id", id.as_str())).collect()
}

pub(crate) async fn listed_actors(
    client: &Client,
    access: &BoardAccess,
    asked_ids: &[String],
) -> Option<Vec<Actor>> {
    let mut address = Url::parse(&format!("{}/actors", access.url())).ok()?;
    address
        .query_pairs_mut()
        .extend_pairs(actors_query(asked_ids));
    let response = client
        .get(address)
        .bearer_auth(access.token())
        .send()
        .await
        .ok()?;
    let accepted_response = response.error_for_status().ok()?;
    let listed: ListedActors = accepted_response.json().await.ok()?;
    Some(listed.actors)
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

pub fn reading_client(longest_silence: Duration) -> Option<Client> {
    Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .read_timeout(longest_silence)
        .no_proxy()
        .redirect(Policy::none())
        .build()
        .ok()
}

pub async fn board_still_answers(access: &BoardAccess) -> bool {
    let Some(client) = client() else {
        return false;
    };
    discovery_names_its_own_url(&client, access).await && person_on(&client, access).await.is_some()
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

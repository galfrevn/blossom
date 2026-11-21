use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use serde_json::value::RawValue;

use super::MessageView;
use super::actors::Actor;
use super::tasks::{TASK_EVENT_PREFIX, TaskEvent};

#[derive(Deserialize)]
struct FeedEvent {
    #[serde(rename = "type")]
    event_type: String,
    message: Option<Box<RawValue>>,
    actor: Option<Actor>,
}

#[derive(Default)]
pub struct Batch {
    pub(super) views: Vec<MessageView>,
    pub(super) task_events: Vec<TaskEvent>,
    pub(super) actors: BTreeMap<String, Actor>,
    pub(super) unlisted_actor_ids: Vec<String>,
}

impl Batch {
    pub fn from_views(views: Vec<MessageView>) -> Batch {
        let mut batch = Batch::default();
        for view in views {
            batch.add_view(view);
        }
        batch
    }

    pub fn add_event(&mut self, event_json: &str) -> Result<(), serde_json::Error> {
        let event: FeedEvent = serde_json::from_str(event_json)?;
        if let Some(message) = event.message {
            self.add_view(MessageView::parse(message.get())?);
        }
        if event.event_type.starts_with(TASK_EVENT_PREFIX) {
            let task_event = TaskEvent::parse(event_json, &event.event_type, event.actor.as_ref())?;
            self.task_events.push(task_event);
        }
        if let Some(actor) = event.actor {
            self.keep_latest_details_of(actor);
        }
        Ok(())
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.views.is_empty() && self.task_events.is_empty() && self.actors.is_empty()
    }

    pub(crate) fn actor_ids_without_details(&self) -> Vec<String> {
        let mut named_ids: BTreeSet<&str> = BTreeSet::new();
        for task in self.views.iter().filter_map(|view| view.task.as_ref()) {
            named_ids.extend(task.actor_ids());
        }
        for event in &self.task_events {
            named_ids.extend(event.actor_ids());
        }
        named_ids
            .into_iter()
            .filter(|id| !self.actors.contains_key(*id))
            .map(str::to_owned)
            .collect()
    }

    pub(crate) fn add_listed_actors(&mut self, listed_actors: Vec<Actor>, asked_ids: &[String]) {
        for asked_id in asked_ids {
            let listed = listed_actors.iter().any(|actor| actor.id == *asked_id);
            if !listed {
                self.unlisted_actor_ids.push(asked_id.clone());
            }
        }
        for actor in listed_actors {
            self.keep_latest_details_of(actor);
        }
    }

    fn add_view(&mut self, mut view: MessageView) {
        if let Some(author) = view.author.take() {
            self.keep_latest_details_of(author);
        }
        self.views.push(view);
    }

    fn keep_latest_details_of(&mut self, actor: Actor) {
        self.actors.insert(actor.id.clone(), actor);
    }
}

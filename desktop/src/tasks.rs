use std::sync::atomic::Ordering;
use std::sync::{Arc, PoisonError};

use blossom::contract::{TaskChange, TaskCount, TaskNarrowing, TaskWindow};
use blossom::reader::ReplicaReader;
use tauri::State;
use tauri::ipc::Channel;

use crate::{Session, forward};

impl Session {
    pub(crate) fn catch_up_with_the_tasks(&self, reader: &mut ReplicaReader) {
        let tasks_changed = reader.catch_up_tasks().unwrap_or_else(|error| {
            eprintln!("blossom: the replica's changed tasks could not be read: {error}");
            false
        });
        let nothing_announced_yet = self.tasks.borrow().is_none();
        if !(tasks_changed || nothing_announced_yet) {
            return;
        }
        let revision = self
            .task_revision
            .fetch_add(1, Ordering::SeqCst)
            .wrapping_add(1);
        self.tasks.send_replace(Some(TaskChange { revision }));
    }

    fn read_tasks(
        &self,
        narrowing: &TaskNarrowing,
        start: u32,
        count: u32,
    ) -> Result<TaskWindow, String> {
        let reader = self.reader.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(reader) = reader.as_ref() else {
            return Err(String::from("no replica is open"));
        };
        reader
            .task_window_in(narrowing, start, count)
            .map_err(|error| error.to_string())
    }

    fn read_task_counts(&self, channel: Option<&str>) -> Result<Vec<TaskCount>, String> {
        let reader = self.reader.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(reader) = reader.as_ref() else {
            return Err(String::from("no replica is open"));
        };
        Ok(reader.task_counts_in(channel))
    }
}

#[tauri::command]
pub(crate) fn subscribe_to_tasks(channel: Channel<TaskChange>, session: State<'_, Arc<Session>>) {
    let receiver = session.tasks.subscribe();
    let forwarding = tauri::async_runtime::spawn(forward(channel, receiver));
    session.forwarding_tasks.replace(forwarding);
}

#[tauri::command]
pub(crate) fn read_tasks(
    narrowing: TaskNarrowing,
    start: u32,
    count: u32,
    session: State<'_, Arc<Session>>,
) -> Result<TaskWindow, String> {
    session.read_tasks(&narrowing, start, count)
}

#[tauri::command]
pub(crate) fn read_task_counts(
    channel: Option<String>,
    session: State<'_, Arc<Session>>,
) -> Result<Vec<TaskCount>, String> {
    session.read_task_counts(channel.as_deref())
}

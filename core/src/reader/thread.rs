use rusqlite::params;

use crate::contract::{Thread, ThreadMessage};
use crate::reader::ReplicaReader;
use crate::replica::ReplicaError;

impl ReplicaReader {
    pub fn thread_of(&self, message_id: &str) -> Result<Option<Thread>, ReplicaError> {
        let mut statement = self.connection.prepare_cached(
            "SELECT thread,
                    channel,
                    id,
                    created_at,
                    coalesce(json_extract(view, '$.author.name'), json_extract(view, '$.by.actor'), ''),
                    json_extract(view, '$.author.via.name'),
                    coalesce(json_extract(view, '$.text'), ''),
                    json_extract(view, '$.editedAt') IS NOT NULL,
                    coalesce(json_extract(view, '$.deleted'), 0)
             FROM messages
             WHERE thread = (SELECT thread FROM messages WHERE id = ?1)
             ORDER BY created_at, id",
        )?;
        let rows = statement.query_map(params![message_id], |row| {
            let thread_id: String = row.get(0)?;
            let channel: String = row.get(1)?;
            let message = ThreadMessage {
                id: row.get(2)?,
                created_at: row.get(3)?,
                author: row.get(4)?,
                on_behalf_of: row.get(5)?,
                text: row.get(6)?,
                edited: row.get(7)?,
                deleted: row.get(8)?,
            };
            Ok((thread_id, channel, message))
        })?;
        let mut thread_and_channel = None;
        let mut messages = Vec::new();
        for row in rows {
            let (thread_id, channel, message) = row?;
            thread_and_channel.get_or_insert((thread_id, channel));
            messages.push(message);
        }
        let Some((id, channel)) = thread_and_channel else {
            return Ok(None);
        };
        Ok(Some(Thread {
            id,
            channel,
            messages,
        }))
    }
}

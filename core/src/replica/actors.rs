use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Actor {
    pub id: String,
    pub kind: String,
    pub handle: String,
    pub name: String,
    pub via: Option<Via>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Via {
    pub name: String,
    pub environment: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct AuthorKeys {
    pub(crate) actor: String,
}

#[derive(Deserialize)]
pub(crate) struct ActorDetails {
    kind: String,
    handle: String,
    name: String,
    via: Option<Via>,
}

impl ActorDetails {
    pub(crate) fn kind(&self) -> &str {
        &self.kind
    }

    pub(crate) fn handle(&self) -> &str {
        &self.handle
    }

    pub(crate) fn of_actor(self, id: String) -> Actor {
        Actor {
            id,
            kind: self.kind,
            handle: self.handle,
            name: self.name,
            via: self.via,
        }
    }
}

pub(super) fn keep_actors<'batch>(
    transaction: &Transaction<'_>,
    actors: impl Iterator<Item = &'batch Actor>,
) -> rusqlite::Result<()> {
    let mut keep_latest_details = transaction.prepare_cached(
        "INSERT INTO actors (id, kind, handle, name, via_name, via_environment)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT (id) DO UPDATE SET
             kind = excluded.kind,
             handle = excluded.handle,
             name = excluded.name,
             via_name = excluded.via_name,
             via_environment = excluded.via_environment",
    )?;
    for actor in actors {
        let via_name = actor.via.as_ref().map(|via| via.name.as_str());
        let via_environment = actor
            .via
            .as_ref()
            .and_then(|via| via.environment.as_deref());
        keep_latest_details.execute(params![
            actor.id,
            actor.kind,
            actor.handle,
            actor.name,
            via_name,
            via_environment
        ])?;
    }
    Ok(())
}

pub(super) fn keep_unlisted_actors(
    transaction: &Transaction<'_>,
    unlisted_ids: &[String],
) -> rusqlite::Result<()> {
    let mut remember_unlisted = transaction.prepare_cached(
        "INSERT INTO unlisted_actors (id) VALUES (?1) ON CONFLICT (id) DO NOTHING",
    )?;
    for id in unlisted_ids {
        remember_unlisted.execute(params![id])?;
    }
    Ok(())
}

pub(super) fn actor(connection: &Connection, id: &str) -> rusqlite::Result<Option<Actor>> {
    connection
        .prepare_cached(
            "SELECT id, kind, handle, name, via_name, via_environment FROM actors WHERE id = ?1",
        )?
        .query_row(params![id], |row| {
            let via_name: Option<String> = row.get(4)?;
            let via_environment: Option<String> = row.get(5)?;
            let via = via_name.map(|name| Via {
                name,
                environment: via_environment,
            });
            Ok(Actor {
                id: row.get(0)?,
                kind: row.get(1)?,
                handle: row.get(2)?,
                name: row.get(3)?,
                via,
            })
        })
        .optional()
}

pub(super) fn unseen_actors(
    connection: &Connection,
    named_ids: &[String],
) -> rusqlite::Result<Vec<String>> {
    let mut already_answered = connection.prepare_cached(
        "SELECT EXISTS (SELECT 1 FROM actors WHERE id = ?1)
             OR EXISTS (SELECT 1 FROM unlisted_actors WHERE id = ?1)",
    )?;
    let mut unseen_ids = Vec::new();
    for id in named_ids {
        let answered: bool = already_answered.query_row(params![id], |row| row.get(0))?;
        if !answered {
            unseen_ids.push(id.clone());
        }
    }
    Ok(unseen_ids)
}

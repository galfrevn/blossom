use std::ops::Range;

use rusqlite::{Connection, OptionalExtension, params};

use crate::contract::MentionedActor;

const MENTION_START: &str = "@[";
const MENTION_END: char = ']';
const ANOTHER_MENTION_START: char = '[';

pub(super) struct Mention<'text> {
    pub(super) span: Range<usize>,
    pub(super) id: &'text str,
}

fn ends_an_id(character: char) -> bool {
    character == MENTION_END || character == ANOTHER_MENTION_START || character.is_whitespace()
}

pub(super) fn mentions_in(text: &str) -> Vec<Mention<'_>> {
    let mut mentions = Vec::new();
    let mut searched_from = 0;
    while let Some(found) = text
        .get(searched_from..)
        .and_then(|rest| rest.find(MENTION_START))
    {
        let id_start = searched_from + found + MENTION_START.len();
        let rest = text.get(id_start..).unwrap_or_default();
        let id_length = rest.find(ends_an_id).unwrap_or(rest.len());
        let is_closed = rest
            .get(id_length..)
            .is_some_and(|after| after.starts_with(MENTION_END));
        let id = rest.get(..id_length).unwrap_or_default();
        if id.is_empty() || !is_closed {
            searched_from = id_start;
            continue;
        }
        let end = id_start + id_length + MENTION_END.len_utf8();
        mentions.push(Mention {
            span: searched_from + found..end,
            id,
        });
        searched_from = end;
    }
    mentions
}

fn name_of(connection: &Connection, id: &str) -> rusqlite::Result<Option<String>> {
    connection
        .prepare_cached("SELECT name FROM actors WHERE id = ?1")?
        .query_row(params![id], |row| row.get(0))
        .optional()
}

pub(super) fn actors_mentioned_in(
    connection: &Connection,
    text: &str,
) -> rusqlite::Result<Vec<MentionedActor>> {
    let mut looked_up: Vec<&str> = Vec::new();
    let mut actors = Vec::new();
    for mention in mentions_in(text) {
        if looked_up.contains(&mention.id) {
            continue;
        }
        looked_up.push(mention.id);
        if let Some(name) = name_of(connection, mention.id)? {
            actors.push(MentionedActor {
                id: mention.id.to_owned(),
                name,
            });
        }
    }
    Ok(actors)
}

pub(super) fn reading_of(
    connection: &Connection,
    mention: &Mention<'_>,
) -> rusqlite::Result<String> {
    let name = name_of(connection, mention.id)?;
    Ok(format!("@{}", name.as_deref().unwrap_or(mention.id)))
}

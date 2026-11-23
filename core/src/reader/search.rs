use std::ops::Range;

use rusqlite::{Connection, OptionalExtension, params};

use crate::contract::{FoundAgent, FoundChannel, FoundMessage, FoundTask, SearchResults, TextPart};
use crate::reader::mentions::{Mention, mentions_in, reading_of};

const MOST_RESULTS_IN_A_GROUP: i64 = 50;
const NEWEST_MATCHES_RANKED: i64 = 10_000;
const WORDS_IN_A_SNIPPET: i64 = 12;
const MARK_START: &str = "\u{2}";
const MARK_END: &str = "\u{3}";
const ELLIPSIS: &str = "…";
const QUOTE: char = '"';

pub(super) fn search(connection: &Connection, query: &str) -> rusqlite::Result<SearchResults> {
    let Some(match_expression) = match_expression_of(query) else {
        return Ok(SearchResults::default());
    };
    let snapshot = connection.unchecked_transaction()?;
    let results = SearchResults {
        messages: found_messages(&snapshot, &match_expression)?,
        tasks: found_tasks(&snapshot, &match_expression)?,
        channels: found_channels(&snapshot, &match_expression)?,
        agents: found_agents(&snapshot, &match_expression)?,
    };
    snapshot.commit()?;
    Ok(results)
}

fn match_expression_of(query: &str) -> Option<String> {
    let segments: Vec<&str> = query.split(QUOTE).collect();
    let last_index = segments.len().saturating_sub(1);
    let last_quote_is_open = segments.len().is_multiple_of(2);
    let mut terms = Vec::new();
    for (index, segment) in segments.iter().enumerate() {
        let words: Vec<&str> = segment.split_whitespace().collect();
        let is_quoted = !index.is_multiple_of(2);
        if !is_quoted {
            terms.extend(words.into_iter().map(as_prefix));
            continue;
        }
        if words.is_empty() {
            continue;
        }
        let phrase = words.join(" ");
        let still_being_typed = last_quote_is_open && index == last_index;
        if still_being_typed {
            terms.push(as_prefix(&phrase));
        } else {
            terms.push(as_phrase(&phrase));
        }
    }
    if terms.is_empty() {
        return None;
    }
    Some(terms.join(" "))
}

fn as_prefix(words: &str) -> String {
    format!("{QUOTE}{words}{QUOTE}*")
}

fn as_phrase(words: &str) -> String {
    format!("{QUOTE}{words}{QUOTE}")
}

fn oldest_sequence_ranked(
    connection: &Connection,
    query_for_the_newest_matches: &str,
    match_expression: &str,
) -> rusqlite::Result<i64> {
    let oldest_of_the_newest_matches = connection
        .prepare_cached(query_for_the_newest_matches)?
        .query_row(
            params![match_expression, NEWEST_MATCHES_RANKED - 1],
            |row| row.get(0),
        )
        .optional()?;
    let every_match_is_ranked = i64::MIN;
    Ok(oldest_of_the_newest_matches.unwrap_or(every_match_is_ranked))
}

fn found_messages(
    connection: &Connection,
    match_expression: &str,
) -> rusqlite::Result<Vec<FoundMessage>> {
    let oldest_ranked = oldest_sequence_ranked(
        connection,
        "SELECT rowid FROM message_words WHERE message_words MATCH ?1
         ORDER BY rowid DESC LIMIT 1 OFFSET ?2",
        match_expression,
    )?;
    let mut statement = connection.prepare_cached(
        "SELECT messages.id,
                messages.channel,
                coalesce(
                    json_extract(messages.view, '$.author.name'),
                    json_extract(messages.view, '$.by.actor'),
                    ''
                ),
                messages.created_at,
                coalesce(snippet(message_words, 0, ?3, ?4, ?5, ?6), ''),
                coalesce(json_extract(messages.view, '$.text'), '')
         FROM message_words
         JOIN arrivals ON arrivals.sequence = message_words.rowid
         JOIN messages ON messages.id = arrivals.id
         WHERE message_words MATCH ?1 AND message_words.rowid >= ?2
         ORDER BY message_words.rank
         LIMIT ?7",
    )?;
    let found = statement.query_map(
        params![
            match_expression,
            oldest_ranked,
            MARK_START,
            MARK_END,
            ELLIPSIS,
            WORDS_IN_A_SNIPPET,
            MOST_RESULTS_IN_A_GROUP
        ],
        |row| {
            Ok(FoundMessage {
                id: row.get(0)?,
                channel: row.get(1)?,
                author: row.get(2)?,
                created_at: row.get(3)?,
                snippet: snippet_reading_mentions(
                    connection,
                    &row.get::<_, String>(4)?,
                    &row.get::<_, String>(5)?,
                )?,
            })
        },
    )?;
    found.collect()
}

fn found_tasks(
    connection: &Connection,
    match_expression: &str,
) -> rusqlite::Result<Vec<FoundTask>> {
    let oldest_ranked = oldest_sequence_ranked(
        connection,
        "SELECT rowid FROM task_words WHERE task_words MATCH ?1
         ORDER BY rowid DESC LIMIT 1 OFFSET ?2",
        match_expression,
    )?;
    let mut statement = connection.prepare_cached(
        "SELECT tasks.id,
                tasks.channel,
                tasks.state,
                coalesce(snippet(task_words, 0, ?3, ?4, ?5, ?6), ''),
                coalesce(json_extract(messages.view, '$.text'), '')
         FROM task_words
         JOIN arrivals ON arrivals.sequence = task_words.rowid
         JOIN tasks ON tasks.id = arrivals.id
         JOIN messages ON messages.id = tasks.id
         WHERE task_words MATCH ?1 AND task_words.rowid >= ?2
         ORDER BY task_words.rank
         LIMIT ?7",
    )?;
    let found = statement.query_map(
        params![
            match_expression,
            oldest_ranked,
            MARK_START,
            MARK_END,
            ELLIPSIS,
            WORDS_IN_A_SNIPPET,
            MOST_RESULTS_IN_A_GROUP
        ],
        |row| {
            Ok(FoundTask {
                id: row.get(0)?,
                channel: row.get(1)?,
                state: row.get(2)?,
                snippet: snippet_reading_mentions(
                    connection,
                    &row.get::<_, String>(3)?,
                    &row.get::<_, String>(4)?,
                )?,
            })
        },
    )?;
    found.collect()
}

fn found_channels(
    connection: &Connection,
    match_expression: &str,
) -> rusqlite::Result<Vec<FoundChannel>> {
    let mut statement = connection.prepare_cached(
        "SELECT path, highlight(channel_words, 0, ?2, ?3)
         FROM channel_words
         WHERE channel_words MATCH ?1
         ORDER BY rank
         LIMIT ?4",
    )?;
    let found = statement.query_map(
        params![
            match_expression,
            MARK_START,
            MARK_END,
            MOST_RESULTS_IN_A_GROUP
        ],
        |row| {
            Ok(FoundChannel {
                path: row.get(0)?,
                marked_path: marked_parts(&row.get::<_, String>(1)?),
            })
        },
    )?;
    found.collect()
}

fn found_agents(
    connection: &Connection,
    match_expression: &str,
) -> rusqlite::Result<Vec<FoundAgent>> {
    let mut statement = connection.prepare_cached(
        "SELECT id, highlight(agent_words, 1, ?2, ?3), highlight(agent_words, 2, ?2, ?3)
         FROM agent_words
         WHERE agent_words MATCH ?1
         ORDER BY rank
         LIMIT ?4",
    )?;
    let found = statement.query_map(
        params![
            match_expression,
            MARK_START,
            MARK_END,
            MOST_RESULTS_IN_A_GROUP
        ],
        |row| {
            Ok(FoundAgent {
                id: row.get(0)?,
                marked_name: marked_parts(&row.get::<_, String>(1)?),
                marked_handle: marked_parts(&row.get::<_, String>(2)?),
            })
        },
    )?;
    found.collect()
}

fn marked_parts(marked_text: &str) -> Vec<TextPart> {
    let mut parts = Vec::new();
    let mut rest = marked_text;
    while let Some((before, from_the_mark)) = rest.split_once(MARK_START) {
        push_part(&mut parts, before, false);
        let (marked, after) = from_the_mark
            .split_once(MARK_END)
            .unwrap_or((from_the_mark, ""));
        push_part(&mut parts, marked, true);
        rest = after;
    }
    push_part(&mut parts, rest, false);
    parts
}

fn push_part(parts: &mut Vec<TextPart>, text: &str, marked: bool) {
    if text.is_empty() {
        return;
    }
    parts.push(TextPart {
        text: text.to_owned(),
        marked,
    });
}

struct SnippetText {
    plain: String,
    marked_bytes: Vec<bool>,
}

impl SnippetText {
    fn of(parts: &[TextPart]) -> SnippetText {
        let mut plain = String::new();
        let mut marked_bytes = Vec::new();
        for part in parts {
            plain.push_str(&part.text);
            marked_bytes.resize(plain.len(), part.marked);
        }
        SnippetText {
            plain,
            marked_bytes,
        }
    }
}

struct ShownPart {
    in_snippet: Range<usize>,
    in_text: Range<usize>,
    cut_before: bool,
    cut_after: bool,
}

impl ShownPart {
    fn starting_at_in(
        snippet: &str,
        text: &str,
        cut_before: bool,
        cut_after: bool,
    ) -> Option<ShownPart> {
        let start = if cut_before { ELLIPSIS.len() } else { 0 };
        let end = snippet
            .len()
            .checked_sub(if cut_after { ELLIPSIS.len() } else { 0 })?;
        let shown = snippet.get(start..end)?;
        let found_at = if !cut_before {
            text.starts_with(shown).then_some(0)
        } else if !cut_after {
            text.ends_with(shown).then(|| text.len() - shown.len())
        } else {
            text.find(shown)
        };
        let at = found_at.filter(|at| cut_after || at + shown.len() == text.len())?;
        Some(ShownPart {
            in_snippet: start..end,
            in_text: at..at + shown.len(),
            cut_before,
            cut_after,
        })
    }

    fn of(snippet: &str, text: &str) -> Option<ShownPart> {
        let starts_cut = snippet.starts_with(ELLIPSIS);
        let ends_cut = snippet.len() > ELLIPSIS.len() && snippet.ends_with(ELLIPSIS);
        [
            (starts_cut, ends_cut),
            (false, ends_cut),
            (starts_cut, false),
            (false, false),
        ]
        .into_iter()
        .find_map(|(cut_before, cut_after)| {
            ShownPart::starting_at_in(snippet, text, cut_before, cut_after)
        })
    }

    fn overlaps(&self, mention: &Mention<'_>) -> bool {
        mention.span.start < self.in_text.end && self.in_text.start < mention.span.end
    }
}

fn snippet_reading_mentions(
    connection: &Connection,
    marked_snippet: &str,
    text: &str,
) -> rusqlite::Result<Vec<TextPart>> {
    let as_marked = marked_parts(marked_snippet);
    let snippet = SnippetText::of(&as_marked);
    let Some(shown) = ShownPart::of(&snippet.plain, text) else {
        return Ok(as_marked);
    };
    let mentions: Vec<Mention<'_>> = mentions_in(text)
        .into_iter()
        .filter(|mention| shown.overlaps(mention))
        .collect();
    if mentions.is_empty() {
        return Ok(as_marked);
    }
    let is_marked = |position_in_text: usize| {
        let position_in_snippet = position_in_text - shown.in_text.start + shown.in_snippet.start;
        snippet
            .marked_bytes
            .get(position_in_snippet)
            .copied()
            .unwrap_or(false)
    };
    let mut parts = Vec::new();
    if shown.cut_before {
        push_joined(&mut parts, ELLIPSIS, false);
    }
    let mut position = shown.in_text.start;
    for mention in &mentions {
        let visible =
            mention.span.start.max(shown.in_text.start)..mention.span.end.min(shown.in_text.end);
        push_marked_text(&mut parts, text, position..visible.start, &is_marked);
        let marked = visible.clone().any(&is_marked);
        push_joined(&mut parts, &reading_of(connection, mention)?, marked);
        position = visible.end;
    }
    push_marked_text(&mut parts, text, position..shown.in_text.end, &is_marked);
    if shown.cut_after {
        push_joined(&mut parts, ELLIPSIS, false);
    }
    Ok(parts)
}

fn push_marked_text(
    parts: &mut Vec<TextPart>,
    text: &str,
    range: Range<usize>,
    is_marked: &impl Fn(usize) -> bool,
) {
    let Some(slice) = text.get(range.clone()) else {
        return;
    };
    let mut character_bytes = [0; 4];
    for (offset, character) in slice.char_indices() {
        let marked = is_marked(range.start + offset);
        push_joined(parts, character.encode_utf8(&mut character_bytes), marked);
    }
}

fn push_joined(parts: &mut Vec<TextPart>, text: &str, marked: bool) {
    if let Some(last) = parts.last_mut()
        && last.marked == marked
    {
        last.text.push_str(text);
        return;
    }
    push_part(parts, text, marked);
}

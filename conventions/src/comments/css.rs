use std::iter::Peekable;
use std::str::CharIndices;

use super::LineStarts;

pub fn lines_with_a_comment(source: &str) -> Vec<usize> {
    let line_starts = LineStarts::of(source);
    let mut comment_lines = Vec::new();
    let mut characters = source.char_indices().peekable();
    while let Some((offset, character)) = characters.next() {
        match character {
            '"' | '\'' => skip_string(&mut characters, character),
            '/' if characters.next_if(|(_, next)| *next == '*').is_some() => {
                comment_lines.push(line_starts.line_at(offset));
                skip_comment(&mut characters);
            }
            _ => {}
        }
    }
    comment_lines
}

fn skip_string(characters: &mut Peekable<CharIndices<'_>>, quote: char) {
    while let Some((_, character)) = characters.next() {
        if character == '\\' {
            characters.next();
            continue;
        }
        if character == quote || character == '\n' {
            return;
        }
    }
}

fn skip_comment(characters: &mut Peekable<CharIndices<'_>>) {
    while let Some((_, character)) = characters.next() {
        if character == '*' && characters.next_if(|(_, next)| *next == '/').is_some() {
            return;
        }
    }
}

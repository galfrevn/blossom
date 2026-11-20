mod css;
mod rust;
mod script;

use std::path::Path;

use oxc_span::SourceType;

pub enum Language {
    Rust,
    JavaScriptOrTypeScript(SourceType),
    Css,
}

pub fn language_of(path: &Path) -> Option<Language> {
    let extension = path.extension()?.to_str()?;
    match extension {
        "rs" => Some(Language::Rust),
        "ts" | "tsx" | "mts" | "cts" | "js" | "jsx" | "mjs" | "cjs" => SourceType::from_path(path)
            .ok()
            .map(Language::JavaScriptOrTypeScript),
        "css" => Some(Language::Css),
        _ => None,
    }
}

pub fn lines_with_a_comment(language: &Language, source: &str) -> Vec<usize> {
    match language {
        Language::Rust => rust::lines_with_a_comment(source),
        Language::JavaScriptOrTypeScript(source_type) => {
            script::lines_with_a_comment(source, *source_type)
        }
        Language::Css => css::lines_with_a_comment(source),
    }
}

struct LineStarts {
    offsets: Vec<usize>,
}

impl LineStarts {
    fn of(source: &str) -> LineStarts {
        let offsets_after_each_newline = source
            .match_indices('\n')
            .map(|(newline_offset, _)| newline_offset + 1);
        let offsets = std::iter::once(0)
            .chain(offsets_after_each_newline)
            .collect();
        LineStarts { offsets }
    }

    fn line_at(&self, offset: usize) -> usize {
        self.offsets
            .partition_point(|line_start| *line_start <= offset)
    }
}

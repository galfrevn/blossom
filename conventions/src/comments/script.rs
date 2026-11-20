use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

use super::LineStarts;

pub fn lines_with_a_comment(source: &str, source_type: SourceType) -> Vec<usize> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    let line_starts = LineStarts::of(source);
    parsed
        .program
        .comments
        .iter()
        .map(|comment| line_starts.line_at(comment.span.start as usize))
        .collect()
}

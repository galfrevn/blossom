use ra_ap_rustc_lexer::{FrontmatterAllowed, TokenKind, strip_shebang, tokenize};

pub fn lines_with_a_comment(source: &str) -> Vec<usize> {
    let shebang_length = strip_shebang(source).unwrap_or(0);
    let (shebang, body) = source
        .split_at_checked(shebang_length)
        .unwrap_or(("", source));
    let mut line = 1 + shebang.matches('\n').count();
    let mut remaining_text = body;
    let mut comment_lines = Vec::new();
    for token in tokenize(body, FrontmatterAllowed::Yes) {
        let (token_text, text_after_token) = remaining_text
            .split_at_checked(token.len as usize)
            .unwrap_or((remaining_text, ""));
        let token_is_a_comment = matches!(
            token.kind,
            TokenKind::LineComment { .. } | TokenKind::BlockComment { .. }
        );
        if token_is_a_comment {
            comment_lines.push(line);
        }
        line += token_text.matches('\n').count();
        remaining_text = text_after_token;
    }
    comment_lines
}

use crate::link::link_to_open;

fn opened(link: &str) -> Option<String> {
    link_to_open(link).map(|url| url.to_string())
}

#[test]
fn web_and_mail_links_open_in_the_browser() {
    assert_eq!(
        opened("https://vercel.com/docs"),
        Some("https://vercel.com/docs".to_owned())
    );
    assert_eq!(
        opened("http://www.vercel.com"),
        Some("http://www.vercel.com/".to_owned())
    );
    assert_eq!(
        opened("mailto:ada@example.com"),
        Some("mailto:ada@example.com".to_owned())
    );
}

#[test]
fn a_scheme_in_capitals_is_the_same_scheme() {
    assert_eq!(
        opened("HTTPS://VERCEL.COM"),
        Some("https://vercel.com/".to_owned())
    );
}

#[test]
fn every_other_scheme_is_refused() {
    for link in [
        "javascript:alert(1)",
        "JavaScript:alert(1)",
        " javascript:alert(1)",
        "java\tscript:alert(1)",
        "vbscript:msgbox(1)",
        "data:text/html,<script>alert(1)</script>",
        "file:///Applications/Calculator.app",
        "tauri://localhost/",
        "ftp://example.com/",
        "x-apple.systempreferences:com.apple.preference.security",
    ] {
        assert_eq!(opened(link), None, "{link} should not open");
    }
}

#[test]
fn text_that_is_not_a_whole_link_is_refused() {
    for link in ["", "/threads/2", "#top", "vercel.com", "-a Calculator"] {
        assert_eq!(opened(link), None, "{link} should not open");
    }
}

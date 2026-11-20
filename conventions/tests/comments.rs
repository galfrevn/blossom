mod support;

use std::fs;
use std::path::PathBuf;

use conventions::{Violation, check};
use support::write_tree;

#[test]
fn a_line_comment_in_rust_is_reported_with_its_line() {
    let tree = write_tree(&[(
        "core/replica.rs",
        "fn merge() {}\n// merges two events\nfn apply() {}\n",
    )])
    .unwrap();

    let violations = check(tree.directory.path(), &tree.paths).unwrap();

    assert_eq!(
        violations,
        vec![Violation::Comment {
            path: PathBuf::from("core/replica.rs"),
            line: 2
        }]
    );
}

#[test]
fn doc_comments_are_reported() {
    let tree = write_tree(&[(
        "core/replica.rs",
        "//! the replica\n\n/// merges two events\nfn merge() {}\n",
    )])
    .unwrap();

    let violations = check(tree.directory.path(), &tree.paths).unwrap();

    assert_eq!(
        violations,
        vec![
            Violation::Comment {
                path: PathBuf::from("core/replica.rs"),
                line: 1
            },
            Violation::Comment {
                path: PathBuf::from("core/replica.rs"),
                line: 3
            },
        ]
    );
}

#[test]
fn slashes_inside_strings_are_not_comments() {
    let tree = write_tree(&[(
        "core/board.rs",
        "const LOCAL: &str = \"http://127.0.0.1:4100/api/board\";\nconst RAW: &str = r#\"// \"quoted\" /* */\"#;\n",
    )]).unwrap();

    let violations = check(tree.directory.path(), &tree.paths).unwrap();

    assert_eq!(violations, vec![]);
}

#[test]
fn files_that_are_not_rust_are_not_read() {
    let tree = write_tree(&[("documentation/notes.md", "// a slash pair in prose\n")]).unwrap();
    let bytes_that_are_not_text = [0x89, 0x50, 0x4e, 0x47, 0xff, 0xfe];
    fs::write(
        tree.directory.path().join("documentation/icon.png"),
        bytes_that_are_not_text,
    )
    .unwrap();
    let mut paths = tree.paths.clone();
    paths.push(PathBuf::from("documentation/icon.png"));

    let violations = check(tree.directory.path(), &paths).unwrap();

    assert_eq!(violations, vec![]);
}

#[test]
fn a_rust_file_that_cannot_be_read_is_an_error_that_names_it() {
    let tree = write_tree(&[]).unwrap();
    let paths = vec![PathBuf::from("core/missing.rs")];

    let error = check(tree.directory.path(), &paths).unwrap_err();

    assert_eq!(error.path, PathBuf::from("core/missing.rs"));
}

#[test]
fn a_nested_block_comment_is_reported_once_at_the_line_it_opens() {
    let tree = write_tree(&[(
        "core/replica.rs",
        "fn merge() {}\n\n/* outer\n /* inner */\n*/\nfn apply() {}\n",
    )])
    .unwrap();

    let violations = check(tree.directory.path(), &tree.paths).unwrap();

    assert_eq!(
        violations,
        vec![Violation::Comment {
            path: PathBuf::from("core/replica.rs"),
            line: 3
        }]
    );
}

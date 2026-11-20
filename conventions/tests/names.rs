mod support;

use std::path::PathBuf;

use conventions::{Violation, check};
use support::write_tree;

#[test]
fn a_hyphenated_file_name_is_reported() {
    let tree = write_tree(&[("attention-list.tsx", "")]).unwrap();

    let violations = check(tree.directory.path(), &tree.paths).unwrap();

    assert_eq!(
        violations,
        vec![Violation::NameIsNotOneWord {
            path: PathBuf::from("attention-list.tsx")
        }]
    );
}

#[test]
fn a_capitalised_name_is_reported() {
    let tree = write_tree(&[("attention/List.tsx", "")]).unwrap();

    let violations = check(tree.directory.path(), &tree.paths).unwrap();

    assert_eq!(
        violations,
        vec![Violation::NameIsNotOneWord {
            path: PathBuf::from("attention/List.tsx")
        }]
    );
}

#[test]
fn names_a_tool_dictates_pass() {
    let tree = write_tree(&[
        ("Cargo.toml", ""),
        ("Cargo.lock", ""),
        ("core/Cargo.toml", ""),
        ("README.md", ""),
        ("LICENSE", ""),
        ("CLAUDE.md", ""),
        ("GLOSSARY.md", ""),
        ("desktop/Tauri.toml", ""),
        ("package.json", ""),
        ("bun.lock", ""),
    ])
    .unwrap();

    let violations = check(tree.directory.path(), &tree.paths).unwrap();

    assert_eq!(violations, vec![]);
}

#[test]
fn a_dotfile_named_with_one_word_passes() {
    let tree = write_tree(&[(".gitignore", ""), (".github/workflows/ci.yml", "")]).unwrap();

    let violations = check(tree.directory.path(), &tree.paths).unwrap();

    assert_eq!(violations, vec![]);
}

#[test]
fn third_party_skills_and_the_tracker_are_outside_the_check() {
    let tree = write_tree(&[
        (".claude/skills/to-spec/SKILL.md", ""),
        (".scratch/scaffold/issues/01.md", ""),
    ])
    .unwrap();

    let violations = check(tree.directory.path(), &tree.paths).unwrap();

    assert_eq!(violations, vec![]);
}

#[test]
fn a_misnamed_folder_is_reported_once_whatever_it_holds() {
    let tree = write_tree(&[("src-tauri/main.rs", ""), ("src-tauri/build.rs", "")]).unwrap();

    let violations = check(tree.directory.path(), &tree.paths).unwrap();

    assert_eq!(
        violations,
        vec![Violation::NameIsNotOneWord {
            path: PathBuf::from("src-tauri")
        }]
    );
}

#[test]
fn a_second_word_after_a_dot_is_reported() {
    let tree = write_tree(&[("list.test.tsx", "")]).unwrap();

    let violations = check(tree.directory.path(), &tree.paths).unwrap();

    assert_eq!(
        violations,
        vec![Violation::NameIsNotOneWord {
            path: PathBuf::from("list.test.tsx")
        }]
    );
}

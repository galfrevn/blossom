#![forbid(unsafe_code)]

mod comments;
mod names;

use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Violation {
    NameIsNotOneWord { path: PathBuf },
    Comment { path: PathBuf, line: usize },
}

impl fmt::Display for Violation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Violation::NameIsNotOneWord { path } => {
                write!(
                    formatter,
                    "{}: name is not one lowercase word",
                    path.display()
                )
            }
            Violation::Comment { path, line } => {
                write!(formatter, "{}:{line}: comment", path.display())
            }
        }
    }
}

#[derive(Debug)]
pub struct ReadError {
    pub path: PathBuf,
    pub source: io::Error,
}

impl fmt::Display for ReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}: could not be read: {}",
            self.path.display(),
            self.source
        )
    }
}

impl std::error::Error for ReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

const FOLDERS_OUTSIDE_THE_CHECK: [&str; 2] = [".claude", ".scratch"];

pub fn check(root: &Path, paths: &[PathBuf]) -> Result<Vec<Violation>, ReadError> {
    let mut violations = BTreeSet::new();
    for path in paths.iter().filter(|path| is_inside_the_check(path)) {
        for misnamed_path in names::paths_not_named_with_one_word(path) {
            violations.insert(Violation::NameIsNotOneWord {
                path: misnamed_path,
            });
        }
        if let Some(language) = comments::language_of(path) {
            let source = fs::read_to_string(root.join(path)).map_err(|source| ReadError {
                path: path.clone(),
                source,
            })?;
            for line in comments::lines_with_a_comment(&language, &source) {
                violations.insert(Violation::Comment {
                    path: path.clone(),
                    line,
                });
            }
        }
    }
    Ok(violations.into_iter().collect())
}

fn is_inside_the_check(path: &Path) -> bool {
    let Some(first_component) = path.components().next() else {
        return true;
    };
    !FOLDERS_OUTSIDE_THE_CHECK
        .iter()
        .any(|folder| first_component.as_os_str() == *folder)
}

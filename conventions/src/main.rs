use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use conventions::{Violation, check};

fn main() -> ExitCode {
    match violations_in_this_repository() {
        Ok(violations) if violations.is_empty() => ExitCode::SUCCESS,
        Ok(violations) => {
            for violation in &violations {
                eprintln!("{violation}");
            }
            eprintln!(
                "{} breaks of the conventions in CLAUDE.md",
                violations.len()
            );
            ExitCode::FAILURE
        }
        Err(error) => {
            eprintln!("conventions: {error}");
            ExitCode::from(2)
        }
    }
}

fn violations_in_this_repository() -> Result<Vec<Violation>, Box<dyn Error>> {
    let root = repository_root()?;
    let paths = files_present_that_git_tracks_or_would_track(&root)?;
    Ok(check(&root, &paths)?)
}

fn repository_root() -> Result<PathBuf, Box<dyn Error>> {
    let output = git(Path::new("."), &["rev-parse", "--show-toplevel"])?;
    Ok(PathBuf::from(String::from_utf8(output)?.trim_end()))
}

fn files_present_that_git_tracks_or_would_track(
    root: &Path,
) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let output = git(
        root,
        &[
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ],
    )?;
    let listing = String::from_utf8(output)?;
    let paths = listing
        .split('\0')
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .filter(|path| root.join(path).exists())
        .collect();
    Ok(paths)
}

fn git(directory: &Path, arguments: &[&str]) -> Result<Vec<u8>, Box<dyn Error>> {
    let output = Command::new("git")
        .current_dir(directory)
        .args(arguments)
        .output()?;
    if !output.status.success() {
        let message = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git {} failed: {}", arguments.join(" "), message.trim()).into());
    }
    Ok(output.stdout)
}

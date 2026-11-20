use std::fs;
use std::io;
use std::path::PathBuf;

use tempfile::TempDir;

pub struct Tree {
    pub directory: TempDir,
    pub paths: Vec<PathBuf>,
}

pub fn write_tree(files: &[(&str, &str)]) -> io::Result<Tree> {
    let directory = TempDir::new()?;
    let mut paths = Vec::new();
    for (path, contents) in files {
        let absolute_path = directory.path().join(path);
        if let Some(parent_folder) = absolute_path.parent() {
            fs::create_dir_all(parent_folder)?;
        }
        fs::write(&absolute_path, contents)?;
        paths.push(PathBuf::from(path));
    }
    Ok(Tree { directory, paths })
}

use std::path::{Path, PathBuf};
use std::{fs, io};
use walkdir::WalkDir;

pub fn file_exit_and_not_empty(filepath: &std::path::Path) -> bool {
    filepath.is_file()
        && filepath
            .metadata()
            .map(|meta| meta.len() > 0)
            .unwrap_or(false)
}

#[cfg(unix)]
pub fn create_soft_link(src: &Path, dst: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(src, dst)?;
    Ok(())
}

#[cfg(windows)]
pub fn create_soft_link(src: &Path, dst: &Path) -> io::Result<()> {
    std::os::windows::fs::symlink_file(src, dst)?;
    Ok(())
}

pub async fn rm_empty_folder(dir: &str) -> Result<(), io::Error> {
    let mut empty_dirs = vec![];

    // Iterate over all entries in the directory and any subdirectories
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if entry.path().is_dir() {
            // If the directory is empty, add it to the list of empty directories
            if fs::read_dir(&entry.path()).unwrap().next().is_none() {
                empty_dirs.push(PathBuf::from(&entry.path()));
            }
        }
    }

    // Remove all the empty directories
    for dir in empty_dirs {
        fs::remove_dir(dir).expect("[-]Failed to remove directory");
    }

    Ok(())
}

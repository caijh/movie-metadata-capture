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
    remove_empty_dirs(dir)?;
    Ok(())
}
// remove_empty_dirs() recursively removes all empty directories in the given directory.
// Returns an io::Error if any of the text or directory operations fail.
fn remove_empty_dirs(dir: &str) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            remove_empty_dirs(&path.display().to_string())?;
            if let Ok(entries) = fs::read_dir(&path) {
                if entries.filter_map(|entry| entry.ok()).count() == 0 {
                    fs::remove_dir_all(&path)?;
                }
            }
        }
    }
    Ok(())
}

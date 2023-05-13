use std::path::{Path, PathBuf};
use std::{fs, io};

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
fn remove_empty_dirs(dir_path: &str) -> io::Result<()> {
    let path = Path::new(dir_path);
    if !path.exists() {
        return Ok(());
    }

    // Get a list of all entries in the directory
    let entries = fs::read_dir(dir_path)?;

    for entry in entries {
        let entry = entry?;

        // If the entry is a directory, recursively remove empty directories within it
        if entry.file_type()?.is_dir() {
            remove_empty_dirs(entry.path().to_str().unwrap())?;

            // Check if the directory is now empty and remove it if it is
            if let Ok(entries) = fs::read_dir(entry.path()) {
                if entries.filter_map(|entry| entry.ok()).count() == 0 {
                    fs::remove_dir(entry.path())?;
                }
            }
        }
    }

    Ok(())
}

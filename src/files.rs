use std::io;
use std::path::Path;

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

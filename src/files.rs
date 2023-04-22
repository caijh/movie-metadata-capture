pub fn file_exit_and_not_empty(filepath: &std::path::Path) -> bool {
    filepath.is_file()
        && filepath
            .metadata()
            .map(|meta| meta.len() > 0)
            .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use movie_metadata_capture::config::{get_app_config, load_config_file};

    #[tokio::test]
    async fn test_load_config_file() {
        load_config_file("./Config.toml").await.expect("TODO: panic message");
        let config = get_app_config();
        let sources = &config.sources;
        assert!(sources.len() > 0, "Failed to create test folder!");
    }
}

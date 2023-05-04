#[cfg(test)]
mod tests {
    use movie_metadata_capture::config::AppConfig;
    use movie_metadata_capture::number_parser::get_number;

    #[tokio::test]
    async fn test_get_number() {
        AppConfig::load_config_file("./Config.toml")
            .await
            .expect("");
        let config = AppConfig::get_app_config();
        assert_eq!(
            get_number(&config, "SS016-1.mp4"),
            Some(("SS016-1".to_string(), "".to_string()))
        );
    }
}

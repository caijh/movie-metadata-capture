#[cfg(test)]
mod tests {
    use movie_metadata_capture::config::{get_app_config, load_config_file};

    #[tokio::test]
    async fn test_search_movie() {
        load_config_file("./Config.toml").await.expect("");
        let config = get_app_config();
        let parser = config.sources.get("fanza").unwrap();
        let movie = parser.search("123").await;
        assert!(movie.is_some());
    }
}

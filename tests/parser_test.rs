#[cfg(test)]
mod tests {
    use movie_metadata_capture::config::{get_app_config, load_config_file};
    use movie_metadata_capture::request::set_proxy;

    #[tokio::test]
    async fn test_search_movie() {
        load_config_file("./Config.toml").await.expect("");
        let config = get_app_config();
        if config.proxy.switch {
            set_proxy(&config.proxy).await.expect("fail to set proxy");
        }
        let parser = config.sources.get("fanza").unwrap();
        let movie = parser.search("ka9oae232").await;
        println!("{:?}", movie);
        assert!(movie.is_some());
        assert_eq!("ka9oae232", movie.unwrap().number);
    }
}

#[cfg(test)]
mod tests {
    use movie_metadata_capture::config::{get_app_config, load_config_file};
    use movie_metadata_capture::core::extrafanart_download;
    use movie_metadata_capture::request::set_proxy;
    use movie_metadata_capture::scraping::Scraping;

    #[tokio::test]
    async fn test_scraping_movie() {
        load_config_file("./Config.toml").await.expect("");
        let config = get_app_config();
        if config.proxy.switch {
            set_proxy(&config.proxy).await.expect("fail to set proxy");
        }
        let config = get_app_config();
        let mut scraping = Scraping::new(&config);
        let movie = scraping.search("ka9oae232", None, None).await;
        println!("{:?}", movie);
        assert!(movie.is_some());
        let movie = movie.unwrap();
        assert_eq!("ka9oae232", movie.number);
        let extra_fanart = movie.extrafanart;
        extrafanart_download(extra_fanart, ".", &config).await;
    }
}

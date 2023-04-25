#[cfg(test)]
mod tests {
    use movie_metadata_capture::config::AppConfig;
    use movie_metadata_capture::core::{
        cut_image, download_actor_photo, download_cover, download_extra_fanart,
        download_small_cover, move_subtitles, paste_file_to_folder,
    };
    use movie_metadata_capture::request::set_proxy;
    use movie_metadata_capture::scraping::Scraping;

    #[tokio::test]
    async fn test_scraping_movie() {
        AppConfig::load_config_file("./Config.toml").await.expect("");
        let config = AppConfig::get_app_config();
        if config.proxy.switch {
            set_proxy(&config.proxy).await.expect("fail to set proxy");
        }
        let config = AppConfig::get_app_config();
        let mut scraping = Scraping::new(&config);
        let movie = scraping.search("ka9oae232", None, None).await;
        println!("{:?}", movie);
        assert!(movie.is_some());
        let movie = movie.unwrap();
        assert_eq!("ka9oae232", movie.number);
        let extra_fanart = movie.extrafanart;

        download_small_cover(&movie.cover_small, ".", "./poster.jpg", &config).await;

        download_cover(&movie.cover, ".", "./thumb.jpg", "./fanart.jpg", &config).await;

        download_actor_photo(&movie.actor, ".", "ka9oae232", &config).await;

        download_extra_fanart(extra_fanart, ".", &config).await;

        cut_image(&config, ".", "./thumb.jpg", "./poster.jpg");

        paste_file_to_folder(
            "./extrafanart/extrafanart-1.jpg",
            ".",
            "ka9oae232",
            "",
            "",
            "",
            &config,
        )
        .unwrap();
        move_subtitles("xxx.mp4", ".", "ka9oae232", "", "", "", &config).unwrap();
    }
}

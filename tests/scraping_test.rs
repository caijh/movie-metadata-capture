#[cfg(test)]
mod tests {
    use movie_metadata_capture::config::{get_app_config, load_config_file};
    use movie_metadata_capture::core::{
        cut_image, download_actor_photo, download_cover, download_extra_fanart,
        download_small_cover, paste_file_to_folder,
    };
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

        download_small_cover(&movie.cover_small, ".", "./poster.jpg", &config).await;

        download_cover(&movie.cover, ".", "./thumb.jpg", "./fanart.jpg", &config).await;

        let actor_name = movie.actor_name;
        let actor_photo = movie.actor_photo;
        let joined_vec: Vec<(String, String)> = actor_name
            .into_iter()
            .zip(actor_photo.into_iter())
            .collect();
        download_actor_photo(joined_vec, ".", "ka9oae232", &config).await;

        download_extra_fanart(extra_fanart, ".", &config).await;

        cut_image(&config, ".", "./thumb.jpg", "./poster.jpg");

        // paste_file_to_folder(
        //     "./extrafanart/extrafanart-1.jpg",
        //     ".",
        //     "ka9oae232",
        //     "",
        //     "",
        //     "",
        //     &config,
        // )
        // .unwrap();
    }
}

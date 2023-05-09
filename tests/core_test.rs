#[cfg(test)]
mod tests {
    use movie_metadata_capture::config::AppConfig;
    use movie_metadata_capture::core::{download_cover, download_small_cover};
    use std::path::Path;

    #[tokio::test]
    async fn download_small_cover_test() {
        AppConfig::load_config_file("./Config.toml")
            .await
            .expect("Fail to load config");

        let config = AppConfig::get_app_config();
        let cover_small_url = "https://okami.my.id/wp-content/uploads/2023/04/3JuliaPle3.jpg";
        let dir = "dir";
        let filename = "image.jpg";

        download_small_cover(cover_small_url, dir, filename, &config).await;

        assert!(Path::new(dir).join(filename).exists())
    }

    #[tokio::test]
    async fn download_cover_test() {
        AppConfig::load_config_file("./Config.toml")
            .await
            .expect("Fail to load config");

        let config = AppConfig::get_app_config();

        let cover_url = "https://pics.dmm.co.jp/mono/movie/adult/ka9oae232/ka9oae232pl.jpg";
        let dir = ".";
        let filename = "ka9oae232pl.jpg";
        let fanart_file_name = "fanart-ka9oae232pl.jpg";
        download_cover(cover_url, dir, filename, fanart_file_name, &config).await;

        assert!(Path::new(dir).join(filename).exists())
    }
}

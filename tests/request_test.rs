#[cfg(test)]
mod tests {
    use std::error::Error;
    use url::Url;
    use movie_metadata_capture::config::{get_app_config, load_config_file};
    use movie_metadata_capture::request::{client, get_html_content, set_proxy};

    #[tokio::test]
    async fn test_get_html_content() {
        load_config_file("./Config.toml").await.expect("Fail to load Config.toml, Please check config file.");
        let config = get_app_config();
        if config.proxy.switch {
            set_proxy(&config.proxy).await.expect("Fail to set proxy");
        }

        // Test a valid URL
        // assert!(get_html_content("https://www.google.com").await.is_ok());

        // Test an invalid URL
        // assert!(get_html_content("https://www.nonexistentwebsite.com").await.is_err());
        let detail_url = format!("{}{}","https://www.dmm.co.jp/mono/dvd/-/detail/=/cid=", "ka9oae232");
        let mut url = Url::parse("https://www.dmm.co.jp/age_check/=/declared=yes/").unwrap();
        url.query_pairs_mut()
            .append_pair("rurl", detail_url.as_str());
        let encoded_url = url.as_str();
        println!("{}", encoded_url);
        let result = get_html_content(encoded_url).await;
        let html = if result.is_ok() {
            result.ok()
        } else {
            None
        };
        println!("{:?}", html);
        assert!(html.is_some());
    }


    #[tokio::test]
    async fn test_client() -> Result<(), Box<dyn Error>> {

        load_config_file("./Config.toml").await.expect("Fail to load Config.toml, Please check config file.");
        let config = get_app_config();
        if config.proxy.switch {
            set_proxy(&config.proxy).await?;
        }

        // Call the client function and verify the result
        let _ = client().await?;
        Ok(())
    }

}



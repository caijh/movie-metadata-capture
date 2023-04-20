#[cfg(test)]
mod tests {

    use movie_metadata_capture::request::get_html_content;

    #[tokio::test]
    async fn test_get_html_content() {
        // Test a valid URL
        assert!(get_html_content("https://www.google.com").await.is_ok());

        // Test an invalid URL
        assert!(get_html_content("https://www.nonexistentwebsite.com").await.is_err());

        let url = format!("{}{}","https://www.dmm.co.jp/digital/videoa/-/detail/=/cid=", "AIMS-020");
        let rurl = &urlencoding::encode(url.as_str());
        let url  = format!("{}rurl={}", "https://www.dmm.co.jp/age_check/=/declared=yes/?", rurl);
        let result = get_html_content(url.as_str()).await;
        let html = if result.is_ok() {
            result.ok()
        } else {
            None
        };
        println!("{:?}", html);
        assert!(html.is_some());
    }
}

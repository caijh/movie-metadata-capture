use std::time::Duration;
use std::{
    error::Error,
    fs::{create_dir_all, File},
    io::Write,
    path::PathBuf,
    sync::Arc,
};

use crate::config;
use lazy_static::lazy_static;
use reqwest::{Client, Proxy, StatusCode, Url};
use tokio::sync::RwLock;

pub struct Request {
    client: Client,
}

lazy_static! {
    pub static ref REQUEST: Arc<RwLock<Request>> = {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.0.4896.133 Safari/537.36")
            .cookie_store(true)
            .build()
            .unwrap();
        Arc::new(RwLock::new(Request { client }))
    };
}

impl Request {
    pub async fn set_proxy(proxy: &config::Proxy) -> Result<(), Box<dyn Error>> {
        let timeout = proxy.timeout;
        let proxy = Proxy::all(&proxy.proxy).unwrap();
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.0.4896.133 Safari/537.36")
            .proxy(proxy).cookie_store(true).connect_timeout(Duration::from_secs(timeout)).build().unwrap();
        let req_clone = REQUEST.clone();
        let mut request = req_clone.write().await;
        request.client = client;
        Ok(())
    }

    pub async fn get_client() -> Result<Client, Box<dyn Error>> {
        let request = REQUEST.read().await;
        let client = &request.client;
        Ok(client.clone())
    }

}


pub async fn download_file(url: &str, save_path: &PathBuf) -> Result<PathBuf, Box<dyn Error>> {
    let url = Url::parse(url).unwrap();
    let client = Request::get_client().await?;
    match client.get(url).send().await {
        Ok(res) => {
            let dir = save_path.parent().unwrap();
            create_dir_all(dir)?;
            let mut file = File::create(save_path).unwrap();
            let _ = file.write_all(&res.bytes().await.unwrap());
            Ok(save_path.clone())
        }
        Err(e) => Err(e.into()),
    }
}

pub async fn parallel_download_files(
    dn_list: Vec<(String, PathBuf)>,
) -> Vec<Result<PathBuf, Box<dyn Error>>> {
    let tasks = dn_list
        .into_iter()
        .map(|(url, save_path)| {
            let url = url;
            let save_path = save_path;
            async move { download_file(&url, &save_path).await }
        })
        .collect::<Vec<_>>();

    let results: Vec<Result<PathBuf, Box<dyn Error>>> = futures::future::join_all(tasks).await;
    results
}

// get html content from url
pub async fn get_html_content(url: &str) -> Result<String, Box<dyn Error>> {
    let client = Request::get_client().await?;
    let res = client.get(url).send().await?;
    if res.status() == StatusCode::NOT_FOUND {
        return Err("404".into());
    }
    let body = res.text().await?;
    Ok(body)
}

use std::{
    error::Error,
    fs::{create_dir_all, File},
    io::Write,
    path::PathBuf,
    sync::Arc,
};

use lazy_static::lazy_static;
use reqwest::{Client, Proxy, Url};
use tokio::sync::RwLock;

pub struct Request {
    client: Client,
}

lazy_static! {
    pub static ref REQUEST: Arc<RwLock<Request>> = {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.0.4896.133 Safari/537.36")
            .build()
            .unwrap();
        Arc::new(RwLock::new(Request { client: client }))
    };
}

pub async fn set_proxy(proxy: String) -> Result<(), Box<dyn Error>> {
    let proxy = Proxy::all(proxy).unwrap();

    let client = Client::builder().proxy(proxy).build().unwrap();
    let req_clone = REQUEST.clone();
    let mut request = req_clone.write().await;
    request.client = client;
    Ok(())
}

pub async fn client() -> Result<Client, Box<dyn Error>> {
    let request = REQUEST.read().await;
    let client = &request.client;
    Ok(client.clone())
}

pub async fn download_file(url: &str, save_path: &PathBuf) -> Result<PathBuf, Box<dyn Error>> {
    let url = Url::parse(url).unwrap();
    let client = client().await?;
    match client.get(url).send().await {
        Ok(res) => {
            let dir = save_path.parent().unwrap();
            create_dir_all(dir)?;
            let mut file = File::create(save_path).unwrap();
            let _ = file.write_all(&mut res.bytes().await.unwrap());
            Ok(save_path.clone())
        }
        Err(e) => Err(e.into()),
    }
}

pub async fn parallel_download_files(
    dn_list: Vec<(String, PathBuf)>,
) -> Vec<Result<PathBuf, Box<dyn std::error::Error>>> {
    let tasks = dn_list
        .into_iter()
        .map(|(url, save_path)| {
            let url = url;
            let save_path = save_path.clone();
            async move { download_file(&url, &save_path).await }
        })
        .collect::<Vec<_>>();

    let results: Vec<Result<PathBuf, Box<dyn Error>>> =
        futures::future::join_all(tasks).await;
    results
}

// get html content from url
pub async fn get_html_content(url: &str) -> Result<String, Box<dyn Error>> {
    let client = client().await?;
    let res = client.get(url).send().await?;
    let body = res.text().await?;
    Ok(body)
}

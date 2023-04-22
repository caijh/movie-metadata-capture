use crate::config::AppConfig;
use crate::files::file_exit_and_not_empty;
use crate::request::{download_file, parallel_download_files};
use crate::scraping::Scraping;
use serde_json::Value;
use std::error::Error;
use std::ops::Not;
use std::path::{Path, PathBuf};
use std::ptr::drop_in_place;
use std::{fs, result};
use urlencoding::encode;

pub async fn core_main(
    file_path: &str,
    custom_number: &str,
    sources: Option<String>,
    specified_source: Option<String>,
    config: &AppConfig,
) -> Result<(), Box<dyn Error>> {
    let mut scraping = Scraping::new(config);
    let movie = scraping
        .search(custom_number, sources, specified_source)
        .await;

    if movie.is_none() {
        // move_failed_folder(file_path, &config);
        return Ok(());
    }
    let movie = movie.unwrap();
    let number = movie.number.as_str();
    // =======================================================================判断-C,-CD后缀
    let mut part = "";
    let multi_part =
        if let Some(cd_match) = regex::Regex::new(r"[-_]CD\d+").unwrap().find(file_path) {
            part = cd_match.as_str().clone();
            true
        } else {
            false
        };
    let mut cn_sub = regex::Regex::new(r"[-_]C(\.\w+$|-\w+)|\d+ch(\.\w+$|-\w+)")
        .unwrap()
        .is_match(file_path)
        || file_path.contains("中文")
        || file_path.contains("字幕");
    let c_word = "-C"; // 中文字幕影片后缀

    // 判断是否无码
    let uncensored = if movie.uncensored {
        true
    } else {
        is_uncensored(number, &config)
    };

    let lower_path = file_path.to_lowercase();
    // 判断是否流出
    let (leak, leak_word) = if lower_path.contains("流出") || lower_path.contains("uncensored") {
        (true, "-无码流出")
    } else {
        (false, "")
    };

    // 判断是否hack
    let (hack, hack_word) = if lower_path.contains("hack") || lower_path.contains("破解") {
        (true, "-hack")
    } else {
        (false, "")
    };

    // 判断是否4k
    let _4k = lower_path.contains("4k");

    let cover = movie.cover;
    let ext = image_ext(Some(cover.as_str()));
    let mut fanart_path = format!("fanart{}", ext);
    let mut poster_path = format!("poster{}", ext);
    let mut thumb_path = format!("thumb{}", ext);
    if config.name_rule.image_naming_with_number {
        fanart_path = format!(
            "{}{}{}{}-fanart{}",
            number, leak_word, c_word, hack_word, ext
        );
        poster_path = format!(
            "{}{}{}{}-poster{}",
            number, leak_word, c_word, hack_word, ext
        );
        thumb_path = format!(
            "{}{}{}{}-thumb{}",
            number, leak_word, c_word, hack_word, ext
        );
    }

    let mut number = String::from(number);
    if multi_part {
        number += "CD1"
    }

    match config.common.main_mode {
        1 => {}
        2 => {}
        3 => {}
        _ => {}
    }

    Ok(())
}

fn is_uncensored(number: &str, config: &AppConfig) -> bool {
    let re_str = r"[\d-]{4,}|\d{6}_\d{2,3}|(cz|gedo|k|n|red-|se)\d{2,4}|heyzo.+|xxx-av-.+|heydouga-.+|x-art\.\d{2}\.\d{2}\.\d{2}";
    let re = regex::Regex::new(re_str).unwrap();
    if re.is_match(number) {
        true
    } else {
        let uncensored_prefix_set: Vec<&str> = config
            .uncensored
            .uncensored_prefix
            .split(',')
            .map(|s| s.trim())
            .collect();
        uncensored_prefix_set.contains(&number)
    }
}

fn image_ext(url: Option<&str>) -> &str {
    let image_extensions = [".jpg", ".jpeg", ".gif", ".png", ".bmp"];
    url.and_then(move |s| {
        for x in image_extensions.iter() {
            if s.ends_with(x) {
                return Some(x.clone());
            }
        }
        None
    })
    .unwrap_or(".jpg")
}

pub async fn download_small_cover(
    cover_small_url: &str,
    dir: &str,
    filename: &str,
    config: &AppConfig,
) {
    let full_filepath = Path::new(dir).join(filename);
    if config.common.download_only_missing_images && file_exit_and_not_empty(&full_filepath) {
        return;
    }
    let ret = download_file_with_filename(cover_small_url, dir, filename, config).await;
    if ret {
        println!(
            "[+]Image Downloaded! {}",
            full_filepath.file_name().unwrap().to_string_lossy()
        );
    }
}

pub async fn download_cover(
    cover_url: &str,
    dir: &str,
    thumb_file_name: &str,
    fanart_file_name: &str,
    config: &AppConfig,
) {
    let full_thumb_path = PathBuf::from(dir).join(thumb_file_name);
    if config.common.download_only_missing_images && file_exit_and_not_empty(&full_thumb_path) {
        return;
    }

    for i in 0..config.proxy.retry {
        let ret = download_file_with_filename(cover_url, dir, thumb_file_name, config).await;
        if ret {
            break;
        }
        println!(
            "[!]Image Download Failed! Trying again. [{}/{}]",
            i + 1,
            config.proxy.retry
        );
    }
    if !file_exit_and_not_empty(&full_thumb_path) {
        return;
    }
    println!(
        "[+]Image Downloaded! {}",
        full_thumb_path.file_name().unwrap().to_string_lossy()
    );
    if !config.common.jellyfin {
        let full_fanart_path = PathBuf::from(dir).join(fanart_file_name);
        match fs::copy(&full_thumb_path, &full_fanart_path) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("[-]Failed to copy thumbnail to fanart: {:?}", e);
            }
        };
    }
}
pub async fn download_file_with_filename(
    url: &str,
    dir: &str,
    filename: &str,
    config: &AppConfig,
) -> bool {
    let full_path = Path::new(dir).join(filename);

    let mut is_success = false;
    for i in 0..config.proxy.retry {
        let result = match download_file(url, &full_path).await {
            Ok(_r) => {
                is_success = true;
                Ok(())
            }
            Err(e) => {
                println!(
                    "[-]Image Download : Proxy error {}/{}",
                    i, config.proxy.retry
                );
                Err(e)
            }
        };
        if result.is_ok() {
            break;
        }
    }
    if !is_success {
        println!("[-]Connect Failed! Please check your Proxy or Network!");
    }

    is_success
}
pub async fn extrafanart_download(extrafanart: Vec<String>, dir: &str, config: &AppConfig) {
    let tm_start = std::time::Instant::now();
    let tasks = extrafanart
        .into_iter()
        .enumerate()
        .map(move |(i, url)| extra_fanart_download_one_by_one(url, i, dir, config))
        .collect::<Vec<_>>();

    futures::future::join_all(tasks).await;
    if config.debug_mode.switch {
        println!(
            "[!]Extrafanart download one by one mode runtime {:.3}s",
            tm_start.elapsed().as_secs_f64()
        );
    }
}

async fn extra_fanart_download_one_by_one(
    extrafanart_url: String,
    i: usize,
    dir: &str,
    config: &AppConfig,
) {
    let extrafanart_path = Path::new(dir).join(&config.extrafanart.extrafanart_folder);
    let download_only_missing_images = config.common.download_only_missing_images;
    let jpg_filename = format!("extrafanart-{}.jpg", i + 1);
    let jpg_full_path = extrafanart_path.join(&jpg_filename);

    for i in 0..config.proxy.retry {
        if download_only_missing_images && file_exit_and_not_empty(&jpg_full_path) {
            break;
        }
        download_file_with_filename(
            extrafanart_url.as_str(),
            extrafanart_path.to_string_lossy().as_ref(),
            &jpg_filename,
            &config,
        )
        .await;
        if !file_exit_and_not_empty(&jpg_full_path) {
            println!(
                "[!]Image Download Failed! Trying again. [{}/{}]",
                i + 1,
                config.proxy.retry
            );
        }
    }
}

pub async fn download_actor_photo(
    actors: Vec<(String, String)>,
    dir: &str,
    number: &str,
    config: &AppConfig,
) {
    if actors.is_empty() || dir.is_empty() {
        return;
    }
    let save_path = Path::new(dir);
    if !save_path.is_dir() {
        return;
    }
    let actors_dir = save_path.join(".actors");
    let download_only_missing_images = config.common.download_only_missing_images;
    let mut dn_list = Vec::new();
    for (actor_name, url) in actors.iter() {
        if url.is_empty().not() {
            let pic_full_path = actors_dir.join(format!("{}{}", actor_name, image_ext(Some(url))));
            if download_only_missing_images && file_exit_and_not_empty(&pic_full_path) {
                continue;
            }
            dn_list.push((url.to_owned(), pic_full_path));
        }
    }
    if dn_list.is_empty() {
        return;
    }
    let result = parallel_download_files(dn_list).await;
    let mut failed = 0;
    for (_i, r) in result.iter().enumerate() {
        if r.is_err() {
            failed += 1;
        }
    }
    if failed > 0 {
        println!(
            "[-]Failed downloaded {}/{} actor photo for [{}] to '{}', you may retry run mode 3 later.",
            failed,
            result.len(),
            number,
            actors_dir.display()
        );
    } else {
        println!("[+]Successfully downloaded {} actor photo.", result.len());
    }
}

use crate::config::{AppConfig, NumberExtractor};
use crate::files::{create_soft_link, file_exit_and_not_empty};
use crate::request::{download_file, parallel_download_files};
use crate::scraping::Scraping;
use std::collections::HashSet;

use std::error::Error;
use std::fs::{hard_link, OpenOptions};

use std::ops::Not;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::parser::Movie;
use dlib_face_recognition::{
    FaceDetector, FaceDetectorCnn, FaceDetectorTrait, FaceLocations, ImageMatrix,
};

use chrono::Local;
use image::{open, DynamicImage};
use quick_xml::se::to_string;
use serde::Serialize;
use std::fs;
use std::io::Write;
use xmlem::{Document, display};

use crate::number_parser::get_number;
use walkdir::WalkDir;

pub async fn core_main(
    file_path: &str,
    number_extractor: &NumberExtractor,
    custom_number: &str,
    sources: Option<String>,
    specified_source: Option<String>,
    config: &AppConfig,
) -> Result<(), Box<dyn Error>> {
    let mut scraping = Scraping::new(config);
    let movie = scraping
        .search(custom_number, number_extractor, sources, specified_source)
        .await;

    if movie.is_none() {
        move_failed_folder(file_path, &config);
        return Ok(());
    }
    let movie = movie.unwrap();
    let number = &movie.number;
    // =======================================================================判断-C,-CD后缀
    let cn_sub = regex::Regex::new(r"[-_]C(\.\w+$|-\w+)|\d+ch(\.\w+$|-\w+)")
        .unwrap()
        .is_match(file_path)
        || file_path.contains("中文")
        || file_path.contains("字幕");
    let c_word = if cn_sub { "-C" } else { "" }; // 中文字幕影片后缀

    // 判断是否无码
    let uncensored = if movie.uncensored {
        true
    } else {
        is_uncensored(number, &config)
    };

    let lower_path = file_path.to_lowercase();
    // 判断是否流出
    let (_leak, leak_word) = if lower_path.contains("流出") || lower_path.contains("uncensored") {
        (true, "-无码流出")
    } else {
        (false, "")
    };

    // 判断是否hack
    let (_hack, hack_word) = if lower_path.contains("hack") || lower_path.contains("破解") {
        (true, "-hack")
    } else {
        (false, "")
    };

    // 判断是否4k
    let _4k = lower_path.contains("4k");

    let cover = &movie.cover;
    let ext = image_ext(Some(cover.as_str()));
    let mut thumb_path = format!("thumb{}", ext);
    let mut poster_path = format!("poster{}", ext);
    let mut fanart_path = format!("fanart{}", ext);
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

    match config.common.main_mode {
        1 => {
            // 创建文件夹
            let path = create_folder(&movie, config);
            let path_str = path.to_string_lossy();
            let dir = path_str.as_ref();

            if movie.cover_small.is_empty().not() {
                download_small_cover(&movie.cover_small, dir, &poster_path, config).await;
            }

            let cover = movie.cover.clone();
            download_cover(&cover, dir, &thumb_path, &fanart_path, config).await;

            if config.extra_fanart.switch {
                let extra_fanart = &movie.extra_fanart;
                download_extra_fanart(extra_fanart, dir, config).await;
            }

            download_actor_photo(&movie.actor, dir, &number, config).await;

            if movie.cover_small.is_empty() {
                cut_image(&config, dir, &thumb_path, &poster_path);
            }

            paste_file_to_folder(
                file_path,
                dir,
                number.as_str(),
                leak_word,
                c_word,
                hack_word,
                config,
            )
            .await?;

            write_nfo_file(
                config,
                &movie,
                dir,
                leak_word,
                c_word,
                hack_word,
                _4k,
                uncensored,
                file_path,
                &thumb_path,
                &poster_path,
                &fanart_path,
            )
            .await?;
        }
        2 => {
            // 创建文件夹
            let path = create_folder(&movie, config);
            let path_str = path.to_string_lossy();
            let dir = path_str.as_ref();
            paste_file_to_folder(
                file_path,
                dir,
                number.as_str(),
                leak_word,
                c_word,
                hack_word,
                config,
            )
            .await?;
        }
        3 => {
            // 创建文件夹
            let path = create_folder(&movie, config);
            let path_str = path.to_string_lossy();
            let dir = path_str.as_ref();

            if movie.cover_small.is_empty().not() {
                download_small_cover(&movie.cover_small, dir, &poster_path, config).await;
            }

            let cover = movie.cover.clone();
            download_cover(&cover, dir, &thumb_path, &fanart_path, config).await;

            if config.extra_fanart.switch {
                let extra_fanart = &movie.extra_fanart;
                download_extra_fanart(extra_fanart, dir, config).await;
            }

            download_actor_photo(&movie.actor, dir, &number, config).await;

            if movie.cover_small.is_empty() {
                cut_image(&config, dir, &thumb_path, &poster_path);
            }

            write_nfo_file(
                config,
                &movie,
                dir,
                leak_word,
                c_word,
                hack_word,
                _4k,
                uncensored,
                file_path,
                &thumb_path,
                &poster_path,
                &fanart_path,
            )
            .await?;
        }
        _ => {}
    }

    Ok(())
}

fn create_folder(movie: &Movie, config: &AppConfig) -> PathBuf {
    let success_folder = config.common.success_output_folder.as_str();
    let actor_names = &movie
        .actor
        .iter()
        .map(|(name, _)| name.to_string())
        .collect::<Vec<String>>()
        .join(", ");
    let mut location_rule = config.name_rule.location_rule.clone();

    if location_rule.contains("$actor") {
        let new_rule = if actor_names.len() > 50 {
            location_rule.replace("$actor", "多人作品")
        } else {
            location_rule.replace("$actor", actor_names.as_str())
        };
        location_rule = new_rule;
    }
    let max_len = config.name_rule.max_title_len;
    let title = &movie.title;
    if config.name_rule.location_rule.contains("title") && title.len() > max_len {
        let title = if title.len() > max_len {
            &title[..max_len]
        } else {
            title.as_str()
        };
        let new_location_rule = location_rule.replace("$title", title);
        location_rule = new_location_rule;
    }
    location_rule = location_rule.replace("$number", &movie.number);
    if location_rule.starts_with("/") {
        location_rule = location_rule[1..].parse().unwrap();
    }
    let mut path = std::path::PathBuf::from(success_folder);
    path = path.join(location_rule.trim());
    if !path.exists() {
        match fs::create_dir_all(&path) {
            Ok(_) => {}
            Err(_) => {
                println!("[ERROR] Fatal error! Can not make folder '{:?}'", path);
            }
        }
    }
    return std::path::PathBuf::from(path);
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
    if cover_small_url.is_empty() {
        return;
    }
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
    if cover_url.is_empty() {
        return;
    }
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
    let full_fanart_path = PathBuf::from(dir).join(fanart_file_name);
    match fs::copy(&full_thumb_path, &full_fanart_path) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("[-]Failed to copy thumbnail to fanart: {:?}", e);
        }
    };
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
pub async fn download_extra_fanart(extrafanart: &Vec<String>, dir: &str, config: &AppConfig) {
    let tm_start = std::time::Instant::now();
    let tasks = extrafanart
        .into_iter()
        .enumerate()
        .map(move |(i, url)| extra_fanart_download_one_by_one(url, i, dir, config))
        .collect::<Vec<_>>();

    futures::future::join_all(tasks).await;
    if config.debug_mode.switch {
        println!(
            "[!]Extrafanart downloaded runtime {:.3}s",
            tm_start.elapsed().as_secs_f64()
        );
    }
}

async fn extra_fanart_download_one_by_one(
    extrafanart_url: &str,
    i: usize,
    dir: &str,
    config: &AppConfig,
) {
    let extrafanart_path = Path::new(dir).join(&config.extra_fanart.extra_fanart_folder);
    let download_only_missing_images = config.common.download_only_missing_images;
    let jpg_filename = format!("extrafanart-{}.jpg", i + 1);
    let jpg_full_path = extrafanart_path.join(&jpg_filename);

    for i in 0..config.proxy.retry {
        if download_only_missing_images && file_exit_and_not_empty(&jpg_full_path) {
            break;
        }
        download_file_with_filename(
            extrafanart_url,
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
    actors: &Vec<(String, String)>,
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

pub async fn paste_file_to_folder(
    filepath: &str,
    dir: &str,
    number: &str,
    leak_word: &str,
    c_word: &str,
    hack_word: &str,
    config: &AppConfig,
) -> Result<(), Box<dyn Error>> {
    let file_path = Path::new(filepath);
    let file_extension = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    let target_path = Path::new(dir).join(format!(
        "{}{}{}{}{}{}",
        number,
        leak_word,
        c_word,
        hack_word,
        ".".to_string(),
        file_extension
    ));
    // 任何情况下都不要覆盖，以免遭遇数据源或者引擎错误导致所有文件得到同一个number，逐一
    // 同名覆盖致使全部文件损失且不可追回的最坏情况
    if target_path.exists() {
        println!("[-]File Exists on destination path, we will never overwriting.");
        return Ok(());
    }
    let link_mode = config.common.link_mode;
    // 如果link_mode 1: 建立软链接 2: 硬链接优先、无法建立硬链接再尝试软链接。
    // 移除原先soft_link=2的功能代码，因默认记录日志，已经可追溯文件来源
    let mut soft_link = false;
    let target_path_clone = target_path.clone();
    if link_mode == 2 {
        // 跨卷或跨盘符无法建立硬链接导致异常，回落到建立软链接
        let metadata_result = hard_link(file_path, target_path);
        if metadata_result.is_err() {
            soft_link = true;
        }
    }
    if link_mode == 1 || soft_link {
        let file_rel_path = file_path.strip_prefix(dir).ok().and_then(|p| p.to_str());
        if file_rel_path.is_some() {
            let symlink_result =
                create_soft_link(Path::new(file_rel_path.unwrap()), &target_path_clone);
            if symlink_result.is_err() {
                create_soft_link(file_path, &target_path_clone).unwrap();
            }
        }
    } else {
        fs::rename(filepath, &target_path_clone)?;
    }
    Ok(())
}

pub fn cut_image(config: &AppConfig, dir: &str, thumb_path: &str, poster_path: &str) {
    let full_path_thumb = Path::new(dir).join(thumb_path);
    let full_path_poster = Path::new(dir).join(poster_path);
    if config.common.download_only_missing_images && file_exit_and_not_empty(&full_path_poster) {
        return;
    }

    let img_result = open(&full_path_thumb);
    let filename = full_path_thumb.file_name().unwrap().to_str().unwrap();
    if let Ok(mut img) = img_result {
        let (width, height) = (img.width(), img.height());
        let poster_image = if (width as f64 / height as f64) > (2.0 / 3.0) {
            // 如果宽度大于2
            let s = face_crop_width(&img, filename, width, height, config);
            img.crop(s.0, s.1, s.2, s.3)
        } else if (width as f64 / height as f64) < (2.0 / 3.0) {
            let s = face_crop_height(&img, filename, width, height, config);
            img.crop(s.0, s.1, s.2, s.3)
        } else {
            img
        };
        if let Err(e) = poster_image.save(&full_path_poster) {
            eprintln!("[-]Cover cut failed! {:?}", e);
        } else {
            println!(
                "[+]Image Cutted! {}",
                full_path_poster.file_name().unwrap().to_string_lossy()
            );
        }
    } else if let Err(e) = img_result {
        eprintln!("[-]Image open failed! {:?}", e);
    }
}

fn face_crop_width(
    image: &DynamicImage,
    filename: &str,
    width: u32,
    height: u32,
    config: &AppConfig,
) -> (u32, u32, u32, u32) {
    let aspect_ratio = config.face.aspect_ratio;
    // 新宽度是高度的2/3
    let crop_width_half = height / 3;
    let locations_model = config.face.locations_model.clone();
    let locations_model = locations_model
        .split(',')
        .filter(|x| !x.is_empty())
        .to_owned();

    for model in locations_model {
        if let Some((center, _top)) = face_center(&image, filename, model) {
            if center < crop_width_half {
                return (0, 0, crop_width_half * aspect_ratio as u32, height);
            }
            let crop_left = center - crop_width_half;
            let crop_right = center + crop_width_half;
            if crop_right > width {
                return (
                    width - crop_width_half * aspect_ratio as u32,
                    0,
                    width,
                    height,
                );
            }
            return (crop_left, 0, crop_right, height);
        }
    }

    println!("[-]Not found face!   {}", filename);
    // 默认靠右切
    (
        width - crop_width_half * aspect_ratio as u32,
        0,
        width,
        height,
    )
}

fn face_crop_height(
    image: &DynamicImage,
    filename: &str,
    width: u32,
    height: u32,
    config: &AppConfig,
) -> (u32, u32, u32, u32) {
    let crop_height = (width as f32 * 3.0 / 2.0).round() as u32;

    let locations_model = config.face.locations_model.clone();
    let locations_model: Vec<&str> = locations_model
        .split(',')
        .filter(|x| !x.is_empty())
        .collect();

    for model in locations_model {
        if let Some((_center, top)) = face_center(&image, filename, model) {
            // 如果找到就跳出循环
            if top > 0 {
                // 头部靠上
                let crop_top = top;
                let crop_bottom = crop_height + top;

                if crop_bottom > height {
                    return (0, 0, width, crop_height);
                }
                return (0, crop_top, width, crop_bottom);
            }
        }
    }

    println!("[-]Not found face!   {}", filename);
    // 默认从顶部向下切割
    (0, 0, width, crop_height)
}

fn face_center(image: &DynamicImage, filename: &str, model: &str) -> Option<(u32, u32)> {
    let rgb_image = image.to_rgb8();
    match model {
        "hog" => {
            println!("[-]Model {} found face {}", model, filename);
            let matrix = ImageMatrix::from_image(&rgb_image);
            let detector = FaceDetector::default();
            let locations = detector.face_locations(&matrix);

            // Get the center of each detected face
            get_face_center(locations)
        }
        "cnn" => {
            println!("[-]Model {} found face {}", model, filename);
            let matrix = ImageMatrix::from_image(&rgb_image);
            let detector = FaceDetectorCnn::default().unwrap();
            let locations = detector.face_locations(&matrix);

            // Get the center of each detected face
            get_face_center(locations)
        }
        _ => None,
    }
}

fn get_face_center(locations: FaceLocations) -> Option<(u32, u32)> {
    let face_centers: Vec<(u32, u32)> = locations
        .iter()
        .map(|detection| {
            let center_x = detection.left + (detection.right - detection.left) / 2;
            let center_y = detection.top + (detection.top - detection.bottom) / 2;
            (center_x as u32, center_y as u32)
        })
        .collect();

    if face_centers.is_empty() {
        None
    } else {
        let face_center: (u32, u32) = face_centers.get(0).unwrap().to_owned();
        Some(face_center)
    }
}

pub fn move_subtitles(
    filepath: &str,
    dir: &str,
    number: &str,
    leak_word: &str,
    c_word: &str,
    hack_word: &str,
    config: &AppConfig,
) -> Result<bool, Box<dyn Error>> {
    let file_path = Path::new(filepath);
    let mut is_success = false;
    let sub_res = &config.media.sub_type;
    let link_mode = config.common.main_mode;

    for entry in fs::read_dir(file_path.parent().unwrap())? {
        let sub_file = entry?.path();
        if sub_file.is_file()
            && sub_res.contains(
                &sub_file
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("")
                    .to_lowercase(),
            )
        {
            if file_path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("")
                .to_lowercase()
                != sub_file
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or("")
                    .to_lowercase()
            {
                continue;
            }
            let sub_target_path = Path::new(dir).join(format!(
                "{}{}{}{}{}",
                number,
                leak_word,
                c_word,
                hack_word,
                sub_file
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("")
            ));

            if link_mode == 1 || link_mode == 2 {
                fs::copy(&sub_file, &sub_target_path)?;
                println!("[+]Sub Copied! {}", sub_target_path.to_string_lossy());
                is_success = true;
            } else {
                fs::rename(&sub_file, &sub_target_path)?;
                println!("[+] Sub Moved!  {:?}", sub_target_path);
                is_success = true;
            }

            if is_success {
                break;
            }
        }
    }
    Ok(is_success)
}

async fn write_nfo_file(
    config: &AppConfig,
    movie: &Movie,
    dir: &str,
    leak_word: &str,
    c_word: &str,
    hack_word: &str,
    _4k: bool,
    uncensored: bool,
    filepath: &str,
    thumb_path: &str,
    poster_path: &str,
    fanart_path: &str,
) -> Result<(), Box<dyn Error>> {
    let nfo_path = if config.common.link_mode == 3 {
        Path::new(&filepath).with_extension("nfo")
    } else {
        Path::new(&dir).join(format!(
            "{}{}{}{}.nfo",
            movie.number, leak_word, c_word, hack_word
        ))
    };
    let _path = Path::new(dir);
    fs::create_dir_all(_path).unwrap();

    let title = config
        .name_rule
        .naming_rule
        .replace("$number", &movie.number)
        .replace("$title", &movie.title.replace(&movie.number, ""));
    let actor = movie
        .actor
        .iter()
        .map(|(name, thumb)| {
            let thumb = if thumb.is_empty().not() {
                format!(".actors/{}{}", name, image_ext(Some(thumb)))
            } else {
                thumb.to_owned()
            };
            Actor {
                name: name.to_string(),
                thumb: thumb.to_string(),
            }
        })
        .collect();

    let mut tag: Vec<Tag> = movie
        .tag
        .iter()
        .map(|tag| Tag {
            content: tag.to_string(),
        })
        .collect();
    if tag.is_empty() {
        if c_word.is_empty().not() {
            tag.push(Tag {
                content: "中文字幕".to_string(),
            })
        }
        if leak_word.is_empty().not() {
            tag.push(Tag {
                content: "流出".to_string(),
            })
        }
        if hack_word.is_empty().not() {
            tag.push(Tag {
                content: "破解".to_string(),
            })
        }
        if uncensored {
            tag.push(Tag {
                content: "无码".to_string(),
            })
        }
    }
    let thumb = Thumb { content: fanart_path.to_string()};
    let fanart = Fanart {
        thumb: vec![thumb],
    };
    let rating = Rating {
        value: movie.user_rating.to_string(),
        votes: movie.user_votes.to_string(),
        max: movie.max_user_rating.to_string(),
        default: true
    };
    let ratings = Ratings {
        rating: vec![rating],
    };
    let nfo = MovieNFO {
        title: title.clone(),
        original_title: movie.title.clone(),
        sort_title: title.clone(),
        customrating: "JP-18+".to_string(),
        mpaa: "JP-18+".to_string(),
        set: movie.series.clone(),
        studio: movie.studio.clone(),
        year: movie.year.clone(),
        outline: movie.outline.clone(),
        plot: movie.outline.clone(),
        runtime: movie.runtime.clone(),
        director: movie.director.clone(),
        poster: poster_path.to_string(),
        thumb: thumb_path.to_string(),
        fanart,
        actors: actor,
        maker: movie.studio.clone(),
        label: movie.label.clone(),
        tag,
        num: movie.number.clone(),
        premiered: movie.release.clone(),
        release_date: movie.release.clone(),
        release: movie.release.clone(),
        userrating: movie.user_rating.clone(),
        ratings,
        cover: movie.cover.clone(),
        trailer: "".to_string(),
        website: movie.website.clone(),
    };

    // write movie to nfo file
    let xml = to_string(&nfo).unwrap();
    let xml = "<?xml version=\"1.0\" encoding=\"UTF-8\" ?>".to_string() + &xml;
    let doc = Document::from_str(&xml).unwrap();
    let dp = display::Config {
        is_pretty: true,
        indent: 2,
        end_pad: 0,
        max_line_length: usize::MAX,
        entity_mode: display::EntityMode::Standard,
        indent_text_nodes: false,
    };
    let xml = doc.to_string_pretty_with_config(&dp);
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&nfo_path)?;
    // Write the XML string to the file
    file.write_all(xml.as_bytes())?;
    println!("[+]Wrote!  {}", &nfo_path.to_string_lossy());
    Ok(())
}

#[derive(Serialize)]
#[serde(rename = "movie")]
struct MovieNFO {
    title: String,
    #[serde(rename = "originaltitle")]
    original_title: String,
    #[serde(rename = "sorttitle")]
    sort_title: String,
    customrating: String,
    mpaa: String,
    set: String,
    studio: String,
    year: String,
    outline: String,
    plot: String,
    runtime: String,
    director: String,
    poster: String,
    thumb: String,
    fanart: Fanart,
    #[serde(rename = "actor")]
    actors: Vec<Actor>,
    maker: String,
    label: String,
    #[serde(rename = "tag")]
    tag: Vec<Tag>,
    num: String,
    premiered: String,
    #[serde(rename = "releasedate")]
    release_date: String,
    release: String,
    userrating: String,
    ratings: Ratings,
    cover: String,
    trailer: String,
    website: String,
}
#[derive(Serialize)]
struct Actor {
    name: String,
    thumb: String,
}

#[derive(Serialize)]
struct Tag {
    #[serde(rename = "$value")]
    content: String,
}

#[derive(Serialize)]
struct Thumb {
    #[serde(rename = "$value")]
    content: String,
}

#[derive(Serialize)]
struct Fanart {
    #[serde(rename = "thumb")]
    thumb: Vec<Thumb>,
}

#[derive(Serialize)]
struct Ratings {
    #[serde(rename = "rating")]
    rating: Vec<Rating>,
}

#[derive(Serialize)]
pub struct Rating {
    pub value: String,
    pub votes: String,
    #[serde(rename = "@max")]
    pub max: String,
    #[serde(rename = "@default")]
    pub default: bool,
}
pub async fn scraping_data_and_move_movie_with_custom_number(
    file_path: &str,
    custom_number: &str,
    number_extractor: &NumberExtractor,
    specified_source: Option<String>,
    config: &AppConfig,
) -> Result<(), Box<dyn Error>> {
    let file_name = Path::new(file_path).file_name().unwrap().to_str().unwrap();

    println!(
        "[!][{}] As Number Processing for '{}'",
        custom_number, file_path
    );
    if !custom_number.is_empty() {
        match core_main(
            file_path,
            number_extractor,
            custom_number,
            None,
            specified_source,
            &config,
        )
        .await
        {
            Ok(r) => r,
            Err(err) => {
                eprintln!("[-] [{}] ERROR:", file_path);
                eprintln!("[-] {}", err);

                if config.common.link_mode > 0 {
                    let link_path =
                        Path::new(config.common.failed_output_folder.as_str()).join(file_name);
                    match create_soft_link(Path::new(file_path), &link_path) {
                        Ok(_) => println!("[-]Link {} to failed folder", file_path),
                        Err(err) => eprintln!("[!] Error while creating symlink - {}", err),
                    };
                } else {
                    let move_path =
                        Path::new(config.common.failed_output_folder.as_str()).join(file_name);
                    match fs::rename(file_path, &move_path) {
                        Ok(_) => println!("[-] Move [{}] to failed folder", file_path),
                        Err(err) => eprintln!("[!] Error while moving file - {}", err),
                    };
                }
            }
        }
    } else {
        println!("[-] number empty ERROR");
    }
    println!("[*]======================================================");
    Ok(())
}

/// Generates list of movies from a given folder.
///
/// This function takes config and folder path as input and searches for all the valid media files present in the folder.
/// It returns a vector of strings which contains paths to the media files.
///
/// # Arguments
///
/// *  `config`  - the AppConfig struct containing source folder and media type
/// *  `folder_path`  - The path of the source folder
///
/// # Returns
///
/// *  `Vec<String>`  - Vector of strings containing paths to the media files.
pub fn movie_lists(config: &AppConfig, folder_path: &Path) -> Vec<String> {
    if !folder_path.is_dir() {
        println!("[-]Source folder not found!");
        return Vec::new();
    }

    let media_type = &config.media.media_type.to_lowercase();
    let file_types: HashSet<&str> = media_type.split(",").collect();
    let mut total_movies: Vec<String> = Vec::new();

    for entry in WalkDir::new(folder_path) {
        let entry = entry.unwrap();
        let path = entry.path();

        if entry.file_type().is_file()
            && path.extension().map_or(false, |ext| {
                file_types
                    .contains((".".to_string() + &ext.to_str().unwrap().to_lowercase()).as_str())
            })
        {
            let movie = path.to_str().unwrap();
            total_movies.push(movie.to_string());
        }
    }

    total_movies
}

pub fn move_failed_folder(filepath: &str, config: &AppConfig) {
    let failed_folder = config.common.failed_output_folder.as_str();
    let link_mode = config.common.link_mode;

    // 模式3或软连接，改为维护一个失败列表，启动扫描时加载用于排除该路径，以免反复处理
    // 原先的创建软连接到失败目录，并不直观，不方便找到失败文件位置，不如直接记录该文件路径
    if config.common.main_mode == 3 || link_mode > 0 {
        let ftxt = std::path::PathBuf::from(failed_folder).join("failed_list.txt");
        println!("-Add to Failed List file, see '{}'", ftxt.display());
        if let Err(e) = fs::write(ftxt, filepath.to_owned() + "\n") {
            eprintln!("-Failed to write failed file to list: {}", e);
        }
    } else if config.common.failed_move && (link_mode == 0) {
        let mut failed_name = std::path::PathBuf::from(failed_folder);
        failed_name.push(Path::new(filepath).file_name().unwrap_or_default());
        let mtxt =
            std::path::PathBuf::from(failed_folder).join("where_was_i_before_being_moved.txt");
        println!("-Move to Failed output folder, see '{}'", mtxt.display());
        if let Err(e) = fs::create_dir_all(mtxt.parent().unwrap()) {
            eprintln!(
                "-Failed to create parent directory of 'where_was_i_before_being_moved.txt': {}",
                e
            );
        }
        if let Ok(mut wwibbmt) = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&mtxt)
        {
            let tmstr = Local::now().format("%Y-%m-%d %H:%M").to_string();
            if let Err(e) = writeln!(
                wwibbmt,
                "{} FROM[{}]TO[{}]",
                tmstr,
                filepath,
                failed_name.display()
            ) {
                eprintln!(
                    "-Failed to write 'where_was_i_before_being_moved.txt': {}",
                    e
                );
            }
        } else {
            eprintln!("-Failed to open 'where_was_i_before_being_moved.txt'.");
        }
        match fs::rename(filepath, &failed_name) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("-File Moving to FailedFolder unsuccessful: {}", e);
            }
        }
    }
}

/// Scraping data and move movie function.
///
/// The parameters  `movie_path`  and  `config`  are used to get the movie number, and move the movie to the appropriate folder.
/// The function checks if the movie number is not empty and if so, will process it and move the movie to the correct folder.
/// If the movie number is empty, it will move the movie to the failed folder.
///
/// # Arguments
///
/// *  `movie_path`  - The path of the movie.
/// *  `config`  - The application configuration.
///
/// # Returns
///
/// A  `Result<(), Box<dyn Error>>`
///
/// # Examples
///
/// scraping_data_and_move_movie("path/to/movie.mp4", &config).await?;
///
pub async fn scraping_data_and_move_movie(
    movie_path: &str,
    config: &AppConfig,
) -> Result<(), Box<dyn Error>> {
    let (n_number, number_extractor) = get_number(config, movie_path).unwrap();
    let movie_path = Path::new(movie_path);
    let movie_path = movie_path.to_string_lossy();
    let movie_path = movie_path.as_ref();
    println!(
        "[!][{}] As Number Processing for '{}'",
        n_number, movie_path
    );
    if n_number.is_empty().not() {
        core_main(
            movie_path,
            &number_extractor,
            n_number.as_str(),
            None,
            None,
            config,
        )
        .await?;
    } else {
        println!("[-] number empty error");
        move_failed_folder(movie_path, config);
    }
    println!("[*]======================================================");
    Ok(())
}

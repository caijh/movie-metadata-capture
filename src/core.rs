use crate::config::AppConfig;
use crate::files::{create_soft_link, file_exit_and_not_empty};
use crate::request::{download_file, parallel_download_files};
use crate::scraping::Scraping;

use std::error::Error;
use std::fs::{hard_link, OpenOptions};

use std::ops::Not;
use std::path::{Path, PathBuf};

use crate::parser::Movie;
use dlib_face_recognition::{
    FaceDetector, FaceDetectorCnn, FaceDetectorTrait, FaceLocations, ImageMatrix,
};
use futures::SinkExt;
use image::{open, DynamicImage};
use quick_xml::se::to_string;
use serde::Serialize;
use std::fs;
use std::io::Write;

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
pub async fn download_extra_fanart(extrafanart: Vec<String>, dir: &str, config: &AppConfig) {
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

pub fn paste_file_to_folder(
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
        return Err("File Exists on destination path, we will never overwriting.".into());
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
        println!("{:?}", (width, height));
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

fn write_nfo_file(
    config: &AppConfig,
    movie: &Movie,
    dir: &str,
    leak_word: &str,
    c_word: &str,
    part: &str,
    filepath: &str,
    hack_word: &str,
    _4k: bool,
    thumb_path: &str,
    poster_path: &str,
    fanart_path: &str,
) -> Result<(), Box<dyn Error>> {
    let nfo_path = if config.common.link_mode == 3 {
        Path::new(&filepath).with_extension("nfo")
    } else {
        Path::new(&dir).join(format!(
            "{}{}{}{}{}.nfo",
            movie.number, part, leak_word, c_word, hack_word
        ))
    };
    let _path = Path::new(dir);
    fs::create_dir_all(_path).unwrap();

    let title = config
        .name_rule
        .naming_rule
        .replace("$number", &movie.number)
        .replace("$title", &movie.title);
    let actor = movie
        .actor
        .iter()
        .map(|(name, thumb)| Actor {
            name: name.to_string(),
            thumb: thumb.to_string(),
        })
        .collect();

    let tag = movie
        .tag
        .iter()
        .map(|tag| Tag {
            content: tag.to_string(),
        })
        .collect();
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
        plot: movie.plot.clone(),
        runtime: movie.runtime.clone(),
        director: movie.director.clone(),
        poster: poster_path.to_string(),
        thumb: thumb_path.to_string(),
        fanart: fanart_path.to_string(),
        actors: actor,
        maker: movie.studio.clone(),
        label: movie.label.clone(),
        tag,
        num: movie.number.clone(),
        premiered: movie.release.clone(),
        releasedate: movie.release.clone(),
        release: movie.release.clone(),
        rating: movie.userrating.clone(),
        cover: movie.cover.clone(),
        trailer: "".to_string(),
        website: movie.website.clone(),
    };

    // write movie to nfo file
    let xml = to_string(&nfo).unwrap();
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
    fanart: String,
    #[serde(rename = "actor")]
    actors: Vec<Actor>,
    maker: String,
    label: String,
    #[serde(rename = "tag")]
    tag: Vec<Tag>,
    num: String,
    premiered: String,
    releasedate: String,
    release: String,
    rating: String,
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

use crate::config::AppConfig;
use lazy_static::lazy_static;
use regex::Regex;
use core::num;
use std::path::Path;

lazy_static! {
    static ref G_SPAT: Regex = Regex::new(r"(?-i)^\w+\.(cc|com|net|me|club|jp|tv|xyz|biz|wiki|info|tw|us|de)@|^22-sht\.me|^((fhd|hd|sd|1080p|720p|4K)(-|_)|(-|_)(fhd|hd|sd|1080p|720p|4K|x264|x265|uncensored|leak))").unwrap();
}

pub fn get_number(config: &AppConfig, file_path: &str) -> Option<(String, String)> {
    let base_name = Path::new(file_path).file_name().unwrap().to_str().unwrap();
    let file_name = G_SPAT.replace_all(base_name, "").to_string();

    for parser in config.number_parser.iter() {
        let number = parser.get_number(&file_name);
        if number.is_some() {
            return number;
        }
    }

    let re_pattern = Regex::new(r#"([^<>/\\|:"*?]+)\.\w+$"#).unwrap();
    let file_name = re_pattern.captures(base_name).map(|c| c[1].to_string())?;
    let number = Regex::new(r"(.+?)\.")
        .unwrap()
        .captures(&file_name)
        .map(|c| c[1].to_string());
    Some((number.unwrap_or_default(), "".to_string()))
}

use crate::config::{AppConfig, NumberExtractor};
use lazy_static::lazy_static;
use regex::Regex;

use std::path::Path;

lazy_static! {
    static ref G_SPAT: Regex = Regex::new(r"(?-i)^\w+\.(cc|com|net|me|club|jp|tv|xyz|biz|wiki|info|tw|us|de)@|^22-sht\.me|^((fhd|hd|sd|1080p|720p|4K)(-|_)|(-|_)(fhd|hd|sd|1080p|720p|4K|x264|x265|uncensored|leak))").unwrap();
    static ref RE_PATTERN: Regex = Regex::new(r#"([^<>/\\|:"*?]+)\.\w+$"#).unwrap();
    static ref NUMBER_REGEX: Regex = Regex::new(r#"\w+(-|_)\w+"#).unwrap();
    pub static ref DEFAULT_NUMBER_EXTRACTOR: NumberExtractor = NumberExtractor {
        name: "default".to_string(),
        regex: r#"\w+(-|_)\w+"#.to_string(),
        sources: None,
    };
}

// This function takes in a configuration and a file path and returns a tuple of strings
pub fn get_number(config: &AppConfig, file_path: &str) -> Option<(String, NumberExtractor)> {
    // Get the base name of the file
    let base_name = Path::new(file_path).file_name().unwrap().to_str().unwrap();
    // Replace all special characters in the base name
    let file_name = G_SPAT.replace_all(base_name, "").to_string();
    // Iterate through the number extractors in the config
    for extractor in config.number_extractor.iter() {
        // Get the number from the file name
        let number = extractor.get_number(&file_name);
        if number.is_some() {
            return Some((number.unwrap(), extractor.to_owned()));
        }
    }

    // Get the first capture group from the regex
    let file_name = RE_PATTERN
        .captures(file_name.as_str())?
        .get(0)?
        .as_str()
        .replace("_", "-");
    // Get the first capture group from the number regex
    let number = NUMBER_REGEX.captures(file_name.as_str())?.get(0)?.as_str();
    Some((number.to_string(), DEFAULT_NUMBER_EXTRACTOR.to_owned()))
}

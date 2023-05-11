use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::{env, fs, io};

use config::{Config, File};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

use glob::glob;

use crate::files::rm_empty_folder;

use crate::request::Request;
use crate::site_search::SiteSearch;
use crate::strings::{get_end_index, get_start_index};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct AppConfig {
    pub common: Common,
    pub proxy: Proxy,
    pub translate: Translate,
    pub number_extractor: Vec<NumberExtractor>,
    pub name_rule: NameRule,
    pub uncensored: Uncensored,
    pub debug_mode: DebugMode,
    pub extra_fanart: ExtraFanart,
    pub face: Face,
    pub media: Media,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SourcesHolder {
    pub sources: HashMap<String, Parser>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Proxy {
    pub switch: bool,
    pub proxy: String,
    pub timeout: u64,
    pub retry: u8,
    pub ca_cert_file: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Translate {
    pub switch: bool,
    pub engine: String,
    pub service_url: String,
    pub access_key: String,
    pub region: Option<String>,
    pub values: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct DebugMode {
    pub switch: bool,
}
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Parser {
    pub name: String,
    pub site_search: Option<SiteSearch>,
    pub number_pre_handle: Vec<NumberHandle>,
    pub source_age_check: Option<AgeCheck>,
    pub source_detail_url: Vec<String>,
    pub source_max_userrating: Option<String>,
    pub source_allow_use_site_number: Option<bool>,

    pub expr_number: String,
    pub expr_title: String,
    pub expr_actor_name: String,
    pub expr_actor_photo: String,
    pub expr_studio: String,
    pub expr_runtime: String,
    pub expr_release: String,
    pub expr_outline: String,
    pub expr_director: String,
    pub expr_tags: String,
    pub expr_label: String,
    pub expr_series: String,
    pub expr_cover: String,
    pub expr_small_cover: String,
    pub expr_extra_fanart: String,
    pub expr_trailer: String,
    pub expr_uncensored: String,
    pub expr_userrating: String,
    pub expr_uservotes: String,

    pub replace_number: Option<Vec<Rule>>,
    pub replace_title: Option<Vec<Rule>>,
    pub replace_outline: Option<Vec<Rule>>,
    pub replace_studio: Option<Vec<Rule>>,
    pub replace_director: Option<Vec<Rule>>,
    pub replace_cover: Option<Vec<Rule>>,
    pub replace_small_cover: Option<Vec<Rule>>,
    pub replace_extra_fanart: Option<Vec<Rule>>,
    pub replace_actor_photo: Option<Vec<Rule>>,
    pub replace_runtime: Option<Vec<Rule>>,
    pub replace_tags: Option<Vec<Rule>>,
    pub replace_label: Option<Vec<Rule>>,
    pub replace_series: Option<Vec<Rule>>,
    pub replace_release: Option<Vec<Rule>>,
    pub replace_userrating: Option<Vec<Rule>>,
    pub replace_uservotes: Option<Vec<Rule>>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct NumberExtractor {
    pub name: String,
    pub regex: String,
    pub sources: Option<Vec<String>>,
}

impl NumberExtractor {
    /// Gets the number from a given filename based on a regex.
    ///
    /// # Arguments
    ///
    /// *  `filename`  - A string representing the filename.
    ///
    /// # Returns
    ///
    /// An Option of a tuple of strings containing the number and the name.
    /// If the regex does not match the filename, None is returned.
    pub fn get_number(&self, filename: &str) -> Option<String> {
        let re = Regex::new(&self.regex).unwrap();
        if re.is_match(filename) {
            let m = re.captures(filename).unwrap();
            return Some(m.get(0).unwrap().as_str().to_string());
        }
        None
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct NameRule {
    pub location_rule: String,
    pub naming_rule: String,
    pub max_title_len: usize,
    pub image_naming_with_number: bool,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Uncensored {
    pub uncensored_prefix: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct AgeCheck {
    pub url: String,
    pub target_name: String,
    pub target_url: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Common {
    pub main_mode: usize,
    pub source_folder: String,
    pub failed_output_folder: String,
    pub success_output_folder: String,
    pub parser_folder: String,
    pub link_mode: usize,
    pub scan_hardlink: bool,
    pub failed_move: bool,
    pub auto_exit: bool,
    pub actor_gender: String,
    pub del_empty_folder: bool,
    pub ignore_failed_list: bool,
    pub download_only_missing_images: bool,
    pub sleep: u64,
}
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ExtraFanart {
    pub switch: bool,
    pub extra_fanart_folder: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Face {
    pub locations_model: String,
    pub aspect_ratio: f32,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Media {
    pub media_type: String,
    pub sub_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NumberHandle {
    pub name: String,
    pub rule: Vec<Rule>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rule {
    pub action: String,
    pub args: Vec<String>,
    pub when: Option<Vec<Condition>>,
}

impl Rule {
    fn is_match(&self, source: &str) -> bool {
        if let Some(when) = &self.when {
            for condition in when {
                if !condition.is_match(source) {
                    return false;
                }
            }
        }
        true
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Condition {
    pub name: String,
    pub args: Vec<String>,
}

impl Condition {
    fn is_match(&self, source: &str) -> bool {
        match self.name.as_str() {
            "contains" => source.contains(self.args[0].as_str()),
            "!contains" => !source.contains(self.args[0].as_str()),
            "empty" => source.is_empty(),
            "!empty" => !source.is_empty(),
            _ => false,
        }
    }
}

// 创建一个数据结构来管理多个操作
pub struct StringFlow {
    rules: Vec<Rule>,
}

impl StringFlow {
    pub fn new(rules: &Vec<Rule>) -> Self {
        let mut _rules = Vec::new();
        for rule in rules {
            _rules.push(rule.clone());
        }
        StringFlow { rules: _rules }
    }

    // 以规则处理字符串
    pub fn process_string(&self, input_string: &str) -> String {
        let mut result = String::from(input_string);

        for rule in self.rules.iter() {
            if !rule.is_match(&result) {
                continue;
            }
            let action = rule.action.as_str();
            match action {
                "append" => {
                    result.push_str(rule.args[0].as_str());
                }
                "replace" => {
                    result = result.replace(rule.args[0].as_str(), rule.args[1].as_str());
                }
                "substring" => {
                    let start = get_start_index(result.as_str(), rule.args[0].as_str());
                    let end = get_end_index(result.as_str(), rule.args[1].as_str());
                    result = result[start..end].to_string();
                }
                "insert" => {
                    let start = get_start_index(result.as_str(), rule.args[0].as_str());
                    let index = result.find(rule.args[1].as_str());
                    if let Some(index) = index {
                        if index != start {
                            result.insert_str(start, rule.args[1].as_str());
                        }
                    } else {
                        result.insert_str(start, rule.args[1].as_str());
                    }
                }
                "between" => {
                    if let Some(start_idx) = result.find(rule.args[0].as_str()) {
                        let start_pos = start_idx + rule.args[0].len();
                        if let Some(end_idx) = result[start_pos..].find(rule.args[1].as_str()) {
                            let end_pos = start_pos + end_idx;
                            result = result[start_pos..end_pos].to_string();
                        }
                    }
                }
                "lowercase" => {
                    result = result.to_lowercase();
                }
                "trim" => {
                    result = result.trim().to_string();
                }
                _ => {}
            }
        }

        result
    }
}

lazy_static! {
    pub static ref CONFIG: Arc<RwLock<AppConfig>> = Arc::new(RwLock::new(AppConfig::default()));
    static ref SOURCES: RwLock<HashMap<String, Parser>> = RwLock::new(HashMap::new());
}

impl AppConfig {
    pub async fn load_config_file(file: &str) -> Result<(), Box<dyn Error>> {
        let settings = Config::builder()
            .add_source(config::File::from(Path::new(file)))
            .build()
            .unwrap_or_else(|_| panic!("[!] Fail to load config file {}", file));
        let cfg = settings.try_deserialize::<AppConfig>().unwrap();

        let config_clone = CONFIG.clone();
        let mut config = config_clone.write().unwrap();
        *config = cfg;

        let parsers = Config::builder()
            .add_source(
                glob((config.common.parser_folder.to_string() + "/*.toml").as_str())
                    .unwrap()
                    .map(|path| File::from(path.unwrap()))
                    .collect::<Vec<_>>(),
            )
            .build()
            .unwrap_or_else(|_| {
                panic!(
                    "[!] Fail to load parsers from dir {}",
                    config.common.parser_folder
                )
            });
        let sources_holder = parsers.try_deserialize::<SourcesHolder>().unwrap();
        for ele in sources_holder.sources {
            SOURCES.write().unwrap().insert(ele.0, ele.1);
        }

        if config.proxy.switch {
            Request::set_proxy(&config.proxy).await?;
        }

        Ok(())
    }

    pub fn get_app_config() -> std::sync::RwLockReadGuard<'static, AppConfig> {
        CONFIG.read().unwrap()
    }

    pub fn get_sources(&self) -> std::sync::RwLockReadGuard<HashMap<String, Parser>> {
        SOURCES.read().unwrap()
    }

    pub fn print_config_and_args(&self) {
        // Print debug status
        if self.debug_mode.switch {
            println!("[+]Enable debug");
        }

        // Print link mode
        match self.common.link_mode {
            1 | 2 => {
                let i = self.common.link_mode - 1;
                println!("[!]Enable {} link", ["soft", "hard"].get(i).unwrap())
            }
            _ => {}
        }

        // Print command-line arguments
        let args: Vec<String> = env::args().collect();
        if args.len() > 1 {
            println!("[!]CmdLine: {}", args[1..].join(" "));
        }

        // Print main working mode
        let main_mode = self.common.main_mode;
        println!(
            "{}",
            format_args!(
                "[+]Main Working mode ## {}: {} ##",
                main_mode,
                ["Scraping", "Organizing", "Scraping in analysis folder"]
                    .get(main_mode - 1)
                    .unwrap(),
            )
        );
    }

    pub async fn create_failed_folder(&self) -> io::Result<()> {
        let failed_folder = &self.common.failed_output_folder;
        match fs::create_dir_all(failed_folder) {
            Ok(_) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => Ok(()),
            Err(error) => {
                println!(
                    "[-]Fatal error! Can not make folder '{}': {}",
                    failed_folder, error
                );
                Err(error)
            }
        }
    }

    pub async fn delete_empty_folder(&self) -> Result<(), io::Error> {
        if self.common.del_empty_folder {
            rm_empty_folder(self.common.success_output_folder.as_str()).await?;
            rm_empty_folder(self.common.source_folder.as_str()).await?;
            rm_empty_folder(self.common.failed_output_folder.as_str()).await?;
        }

        Ok(())
    }
}

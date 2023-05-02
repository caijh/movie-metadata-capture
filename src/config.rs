
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::{env, fs, io};
use std::error::Error;

use crate::files::rm_empty_folder;

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use crate::request::set_proxy;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct AppConfig {
    pub common: Common,
    pub proxy: Proxy,
    pub translate: Translate,
    pub number_extractor: Vec<NumberExtractor>,
    pub sources: HashMap<String, Parser>,
    pub name_rule: NameRule,
    pub uncensored: Uncensored,
    pub debug_mode: DebugMode,
    pub extra_fanart: Extrafanart,
    pub face: Face,
    pub media: Media,
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
    pub number_extractor: Vec<String>,
    pub number_search: Vec<NumberSearch>,
    pub age_check: Option<AgeCheck>,
    pub detail_url: Vec<String>,
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
    pub replace_cover: Option<Vec<Rule>>,
    pub replace_extra_fanart: Option<Vec<Rule>>,
    pub replace_actor_photo: Option<Vec<Rule>>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct NumberExtractor {
    pub name: String,
    pub regex: String,
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
    pub fn get_number(&self, filename: &str) -> Option<(String, String)> {
        let re = Regex::new(&self.regex).unwrap();
        if re.is_match(filename) {
            let m = re.captures(filename).unwrap();
            return Some((m.get(0).unwrap().as_str().to_string(), self.name.to_owned()));
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
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Common {
    pub main_mode: usize,
    pub source_folder: String,
    pub failed_output_folder: String,
    pub success_output_folder: String,
    pub link_mode: usize,
    pub scan_hardlink: bool,
    pub failed_move: bool,
    pub auto_exit: bool,
    pub actor_gender: String,
    pub del_empty_folder: bool,
    pub nfo_skip_days: u8,
    pub ignore_failed_list: bool,
    pub download_only_missing_images: bool,
    pub sleep: u64,
}
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Extrafanart {
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
pub struct NumberSearch {
    pub name: String,
    pub rule: Vec<Rule>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rule {
    pub action: String,
    pub args: Vec<String>,
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

    pub fn add_rules(&mut self, rules: &Vec<Rule>) {
        for rule in rules {
            self.rules.push(rule.clone());
        }
    }

    // 以规则处理字符串
    pub fn process_string(&self, input_string: &str) -> String {
        let mut result = String::from(input_string);

        for rule in self.rules.iter() {
            let action = rule.action.as_str();
            match action {
                "append" => {
                    result.push_str(rule.args[0].as_str());
                }
                "replace" => {
                    result = result.replace(rule.args[0].as_str(), rule.args[1].as_str());
                }
                "substring" => {
                    let start = match rule.args[0].parse::<usize>() {
                        Ok(number) => number,
                        Err(_) => {
                            result.find(&rule.args[0]).unwrap_or_else(|| 0)
                        }
                    };
                    let end = match rule.args[1].parse::<usize>() {
                        Ok(number) => number,
                        Err(_) => {
                            result.len()
                        }
                    };
                    result = result[start..end].to_string();
                }
                "insert" => {
                    let start = match rule.args[0].parse::<usize>() {
                        Ok(number) => number,
                        Err(_) => {
                            result.find(&rule.args[0]).unwrap_or_else(|| 0)
                        }
                    };
                    result.insert_str(start, rule.args[1].as_str());
                }
                "lowercase" => {
                    result = result.to_lowercase();
                }
                _ => {}
            }
        }

        result
    }
}

lazy_static! {
    pub static ref CONFIG: Arc<RwLock<AppConfig>> = Arc::new(RwLock::new(AppConfig::default()));
}

impl AppConfig {
    pub async fn load_config_file(file: &str) -> Result<(), Box<dyn Error>> {
        let config_file_path = PathBuf::from(file);
        let cfg = confy::load_path::<AppConfig>(config_file_path)?;

        let config_clone = CONFIG.clone();
        let mut config = config_clone.write().unwrap();
        *config = cfg;

        if config.proxy.switch {
            set_proxy(&config.proxy).await?;
        }

        Ok(())
    }

    pub fn get_app_config() -> std::sync::RwLockReadGuard<'static, AppConfig> {
        CONFIG.read().unwrap()
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

        let nfo_skip_days = if self.common.nfo_skip_days == 0 {
            "".to_string()
        } else {
            format!(", nfo_skip_days={}", self.common.nfo_skip_days)
        };
        println!(
            "{}",
            format_args!(
                "[+]Main Working mode ## {}: {} ##{}",
                main_mode,
                ["Scraping", "Organizing", "Scraping in analysis folder"]
                    .get(main_mode - 1)
                    .unwrap(),
                nfo_skip_days,
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

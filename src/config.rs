use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use confy::ConfyError;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct AppConfig {
    pub common: Common,
    pub proxy: Proxy,
    pub translate: Translate,
    pub sources: HashMap<String, Parser>,
    pub priority: Priority,
    pub name_rule: NameRule,
    pub uncensored: Uncensored,
    pub debug_mode: DebugMode,
    pub extrafanart: Extrafanart,
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
    pub age_check: Option<AgeCheck>,
    pub detail_url: Vec<String>,
    pub expr_number: String,
    pub expr_title: String,
    pub expr_studio: String,
    pub expr_runtime: String,
    pub expr_release: String,
    pub expr_outline: String,
    pub expr_director: String,
    pub expr_actor: String,
    pub expr_tags: String,
    pub expr_label: String,
    pub expr_series: String,
    pub expr_cover: String,
    pub expr_small_cover: String,
    pub expr_extrafanart: String,
    pub expr_trailer: String,
    pub expr_actorphoto: String,
    pub expr_uncensored: String,
    pub expr_userrating: String,
    pub expr_uservotes: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Priority {
    pub website: String,
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
    pub multi_threading: bool,
    pub actor_gender: String,
    pub del_empty_folder: bool,
    pub nfo_skip_days: u8,
    pub ignore_failed_list: bool,
    pub download_only_missing_images: bool,
    pub mapping_table_validity: u64,
    pub jellyfin: bool,
    pub actor_only_tag: bool,
    pub sleep: u8,
}
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Extrafanart {
    pub switch: bool,
    pub parallel_download: usize,
    pub extrafanart_folder: String,
}
lazy_static! {
    pub static ref CONFIG: Arc<RwLock<AppConfig>> = Arc::new(RwLock::new(AppConfig::default()));
}

pub async fn load_config_file(file: &str) -> Result<(), ConfyError> {
    let config_file_path = PathBuf::from(file);
    let cfg = confy::load_path::<AppConfig>(config_file_path)?;

    let config_clone = CONFIG.clone();
    let mut config = config_clone.write().unwrap();
    *config = cfg;
    Ok(())
}

pub fn get_app_config() -> std::sync::RwLockReadGuard<'static, AppConfig> {
    CONFIG.read().unwrap()
}
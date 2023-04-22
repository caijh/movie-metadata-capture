use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use confy::ConfyError;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct AppConfig {
    pub proxy: Proxy,
    pub translate: Translate,
    pub sources: HashMap<String, Parser>,
    pub priority: Priority,
    pub debug_mode: DebugMode,
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
    pub service_url: String,
    pub access_key: String,
    pub region: Option<String>,
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
pub struct AgeCheck {
    pub url: String,
    pub target_name: String,
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

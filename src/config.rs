use std::collections::HashMap;
use std::path::PathBuf;
use confy::ConfyError;
use lazy_static::lazy_static;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Parser {
    pub source: Source,
    pub expr_number: &'static str,
    pub expr_title: &'static str,
    pub expr_studio: &'static str,
    pub expr_runtime: &'static str,
    pub expr_release: &'static str,
    pub expr_outline: &'static str,
    pub expr_director: &'static str,
    pub expr_actor: &'static str,
    pub expr_tags: &'static str,
    pub expr_label: &'static str,
    pub expr_series: &'static str,
    pub expr_cover: &'static str,
    pub expr_smallcover: &'static str,
    pub expr_extrafanart: &'static str,
    pub expr_trailer: &'static str,
    pub expr_actorphoto: &'static str,
    pub expr_uncensored: &'static str,
    pub expr_userrating: &'static str,
    pub expr_uservotes: &'static str,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Source {
    pub name: String,
    pub detail_url: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct AppConfig {
    pub sources: HashMap<String, Parser>,
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

use clap::{arg, Parser};
use movie_metadata_capture::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();

    let config_path = args.config_path.unwrap_or("./config.toml".to_string());
    println!("[+]Load AppConfig from config file '{}'.", config_path);
    AppConfig::load_config_file(&config_path).await?;

    let config = AppConfig::get_app_config();
    config.print_config_and_args();

    let main_mode = config.common.main_mode;
    if ![1, 2, 3].contains(&main_mode) {
        return Err("[-] Main mode must be 1 or 2 or 3!".into());
    }

    Ok(())
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long, required = false)]
    pub config_path: Option<String>,

    #[arg(long, required = false)]
    pub single_file_path: Option<String>,

    #[arg(long, required = false)]
    pub custom_number: Option<String>,

    #[arg(long, required = false)]
    pub regexstr: Option<String>,

    #[arg(long, required = false)]
    pub log_dir: Option<String>,

    #[arg(long, required = false)]
    pub specified_source: Option<String>,
    #[arg(long, required = false)]
    pub specified_url: Option<String>,
}

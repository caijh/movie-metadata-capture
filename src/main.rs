use std::error::Error;
use std::ops::Not;
use std::path::Path;
use std::sync::RwLockReadGuard;
use std::time::Duration;
use std::{thread, time};

use chrono::Local;
use clap::{arg, Parser, Subcommand};
use rand::Rng;

use movie_metadata_capture::config::AppConfig;
use movie_metadata_capture::core::{
    movie_lists, scraping_data_and_move_movie, scraping_data_and_move_movie_with_custom_number,
};
use movie_metadata_capture::number_parser::{get_number, DEFAULT_NUMBER_EXTRACTOR};
use movie_metadata_capture::scraping::Scraping;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Cli = Cli::parse();

    let config_path = args.config_path.unwrap_or("./config.toml".to_string());
    println!("[+]Load AppConfig from config file '{}'.", config_path);
    AppConfig::load_config_file(&config_path).await?;

    let config: RwLockReadGuard<AppConfig> = AppConfig::get_app_config();
    config.print_config_and_args();

    let check_config_result = config.check_config();
    if check_config_result.is_err() {
        return Err(check_config_result.err().unwrap());
    }

    let start_time = time::Instant::now();
    println!("[+]Start at {}", Local::now().format("%Y-%m-%d %H:%M:%S"));

    match args.subcommand {
        SubCommand::Info(info_args) => {
            let (number, number_extractor) = get_number(&config, info_args.file.as_str()).unwrap();
            println!(
                "[!][{}] As Number Processing for '{}'",
                number, info_args.file
            );
            let mut scraping = Scraping::new(&config);
            let movie = scraping
                .search(&number, &number_extractor, None, Some(info_args.source))
                .await;
            if movie.is_some() && scraping.enable_debug() {
                println!("{:?}", movie);
            }
        }
        SubCommand::Scraping(scraping_args) => {
            config.create_failed_folder().await?;

            let single_file_path = scraping_args.file.unwrap_or_default();
            if !single_file_path.is_empty() {
                println!("[+]==================== Single File =====================");
                let (custom_number, number_extractor) = if scraping_args.custom_number.is_none() {
                    get_number(&config, single_file_path.as_str()).unwrap()
                } else {
                    (
                        scraping_args.custom_number.unwrap_or_default(),
                        DEFAULT_NUMBER_EXTRACTOR.to_owned(),
                    )
                };
                scraping_data_and_move_movie_with_custom_number(
                    &single_file_path,
                    &custom_number,
                    &number_extractor,
                    scraping_args.source,
                    &config,
                )
                .await?;
            } else {
                let folder_path = if config.common.source_folder.is_empty().not() {
                    Path::new(&config.common.source_folder)
                } else {
                    Path::new(".")
                };
                let movie_list = movie_lists(&config, folder_path);

                let movie_count = movie_list.len();
                println!("[+]Find {} movies.", movie_count);
                println!("[*]======================================================");
                for movie_path in movie_list {
                    scraping_data_and_move_movie(movie_path.as_str(), &config).await?;
                    let mut rng = rand::thread_rng();
                    let sleep_seconds = rng.gen_range(config.common.sleep..config.common.sleep + 2);
                    thread::sleep(Duration::from_secs(sleep_seconds));
                }
            }

            config.delete_empty_folder().await?;
        }
    }

    let end_time = time::Instant::now();
    let total_time = end_time.duration_since(start_time);
    let total_time_str = format!("{}", total_time.as_secs_f32());
    println!(
        "[+]Running time {} End at {}",
        total_time_str,
        Local::now().format("%Y-%m-%d %H:%M:%S")
    );

    if config.common.auto_exit {
        std::process::exit(0);
    }

    Ok(())
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(long, required = false)]
    pub config_path: Option<String>,

    #[command(subcommand)]
    subcommand: SubCommand,
}

#[derive(Subcommand, Debug)]
pub enum SubCommand {
    Info(InfoArgs),
    Scraping(ScrapingArgs),
}

#[derive(Parser, Debug)]
pub struct InfoArgs {
    #[arg(long)]
    pub file: String,
    #[arg(long)]
    pub source: String,
}
#[derive(Parser, Debug)]
pub struct ScrapingArgs {
    #[arg(long, required = false)]
    pub file: Option<String>,

    #[arg(long, required = false)]
    pub custom_number: Option<String>,

    #[arg(long, required = false)]
    pub source: Option<String>,
}

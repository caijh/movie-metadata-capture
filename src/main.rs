use chrono::Local;
use clap::{arg, Parser};
use movie_metadata_capture::config::AppConfig;
use movie_metadata_capture::core::{scraping_data_and_move_movie, scraping_data_and_move_movie_with_custom_number, movie_lists};
use movie_metadata_capture::number_parser::get_number;
use rand::Rng;
use std::error::Error;
use std::ops::Not;
use std::path::Path;
use std::time::Duration;
use std::{thread, time};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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

    config.create_failed_folder().await?;

    let start_time = time::Instant::now();
    println!(
        "[+]Start at {}",
        Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
    );

    let single_file_path = args.single_file_path.unwrap_or_default();
    if !single_file_path.is_empty() {
        println!("[+]==================== Single File =====================");
        let (custom_number, number_prefix) = if args.custom_number.is_none() {
            get_number(&config, single_file_path.as_str()).unwrap()
        } else {
            (args.custom_number.unwrap_or_default(), "".to_string())
        };
        scraping_data_and_move_movie_with_custom_number(
            single_file_path.as_str(),
            custom_number.as_str(),
            number_prefix.as_str(),
            args.specified_source,
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

    let end_time = time::Instant::now();
    let total_time = end_time.duration_since(start_time);
    let total_time_str = format!("{}", total_time.as_secs_f32());
    println!(
        "[+]Running time {} End at {}",
        total_time_str,
        Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
    );

    println!("[+]All finished!!!");

    if config.common.auto_exit {
        std::process::exit(0);
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
    pub specified_source: Option<String>,
}

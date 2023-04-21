use std::num;
use std::{iter::Map, collections::HashMap};
use std::error::Error;
use log::debug;
use regex::Regex;
use serde_json::{json, Value};
use crate::config::get_app_config;
use crate::parser::Parser;

#[derive(Default)]
pub struct Scraping {
    adult_full_sources: Vec<String>,
    adult_func_mapping: HashMap<String, Parser>,
    general_full_sources: Vec<String>,
    general_func_mapping: HashMap<String, Parser>,
    specified_source: Option<String>,
    specified_url: Option<String>,
    debug: bool,
}

fn replace_sources_item(mut sources: &Vec<&str>, index: usize, key: &str) {
    let index = sources.iter().position(|s| s.as_ref() == key).unwrap();
    sources.insert(index, sources.remove(index))
}

impl Scraping {
    pub fn new() -> Self {
        let app_config = get_app_config();
        let debug_mode = &app_config.debug_mode.switch;
        let adult_func_mapping = app_config.sources.to_owned();
        Scraping {
            debug: debug_mode.clone(),
            adult_full_sources: vec![
                "javlibrary".to_string(),
                "javdb".to_string(),
                "javbus".to_string(),
                "airav".to_string(),
                "fanza".to_string(),
                "xcity".to_string(),
                "jav321".to_string(),
                "mgstage".to_string(),
                "fc2".to_string(),
                "avsox".to_string(),
                "dlsite".to_string(),
                "carib".to_string(),
                "madou".to_string(),
                "getchu".to_string(),
                "gcolle".to_string(),
                "javday".to_string(),
                "pissplay".to_string(),
                "javmenu".to_string(),
            ],
            adult_func_mapping,
            general_full_sources: vec!["tmdb".to_string(), "imdb".to_string()],
            general_func_mapping: HashMap::new(),
            specified_source: None,
            specified_url: None,
        }
    }

    pub fn search(
        &mut self,
        file_number: &str,
        sources: Option<&str>,
        type_: &str,
        specified_source: Option<&str>,
        specified_url: Option<&str>,
    ) -> Option<Value> {
        self.specified_source = specified_source.map(|s| s.to_string());
        self.specified_url = specified_url.map(|u| u.to_string());
        let sources = sources.unwrap_or("").split(",").collect();

        if type_ == "adult" {
            self.search_adult(file_number, sources)
        } else {
            self.search_general(file_number, sources)
        }
    }

    fn search_adult(&mut self, file_number: &str, sources: Vec<&str>) -> Option<Value> {
        let mut json_data = Value::Null;

        let _sources: Vec<String> = if self.specified_source.is_some() {
            vec![self.specified_source.unwrap()]
        } else {
            self.check_adult_sources(sources, file_number)
        };
        for source in _sources {
            match self.adult_func_mapping.get(source.as_str()) {
                Some(parser) => {
                    if self.debug {
                        println!("[+]select {}", source);
                    }
                    match parser.search(file_number) {
                        Ok(data) => {
                            if data == 404 {
                                continue;
                            }
                            match serde_json::from_str(&data) {
                                Ok(json) => json_data = json,
                                Err(_) => {}
                            }
                        }
                        Err(_) => {}
                    }
                    if self.get_data_state(&json_data) {
                        if self.debug {
                            println!(
                                "[+]Find movie [{}] metadata on website '{}'",
                                file_number, source
                            );
                        }
                        break;
                    }
                }
                None => continue,
            };
        }
        if json_data.is_null() {
            println!("[-]Movie Number [{}] not found!", file_number);
            None
        } else {
            Some(json_data)
        }
    }


    fn search_general(
        &self,
        number: &str,
        mut sources: Vec<&str>,
    ) -> Option<Value> {
        // imdb, tmdb
        if let Some(specified_source) = &self.specified_source {
            sources = vec![specified_source];
        } else {
            sources = self.check_general_sources(sources);
        }
        let mut json_data = json!({});
        for source in sources {
            if self.debug {
                println!("[+]select {}", source);
            }
            match self.general_func_mapping.get(source).map(|f| f(number, self)) {
                Some(Ok(data)) => {
                    if data == "404" {
                        continue;
                    }
                    if let Ok(parsed_json_data) = serde_json::from_str::<Value>(&data) {
                        json_data = parsed_json_data;
                    }
                }
                _ => {}
            }
            // if any service return a valid return, break
            if self.get_data_state(&json_data) {
                if self.debug {
                    println!(
                        "[+]Find movie [{}] metadata on website '{}'",
                        number, source
                    );
                }
                break;
            }
        }

        // Return if data not found in all sources
        if json_data.is_null() {
            eprintln!("[-]Movie Number [{}] not found!", number);
            return None;
        }

        Some(json_data)
    }

    fn check_general_sources(&self, sources: Vec<&str>) -> Vec<&str> {
        sources.into_iter().filter(|&s| self.general_func_mapping.contains_key(s)).collect()
    }

    fn check_adult_sources(&self, sources: Vec<&str>, file_number: &str) -> Vec<&str> {
        let mut _sources: Vec<&str> = if sources.is_empty() {
            self.adult_full_sources.into_iter().map(|s| s.as_str()).collect()
        } else {
            sources
        };
        let lo_file_number = file_number.to_lowercase();
        if _sources.contains(&"carib") && Regex::new(r"^\d{6}-\d{3}").unwrap().is_match(&file_number) {
            replace_sources_item(&sources, 0, "carib")
        } else if file_number.contains("item") || file_number.to_uppercase().contains("GETCHU") {
            replace_sources_item(&sources, 0, "getchu")
        } else if lo_file_number.contains("rj") || lo_file_number.contains("vj") || Regex::new(r"[\u3040-\u309F\u30A0-\u30FF]+").unwrap().is_match(&file_number) {
            replace_sources_item(&sources, 0, "getchu");
            replace_sources_item(&sources, 1, "dlsite")
        } else if lo_file_number.contains("fc2") {
            if sources.contains(&"fc2") {
                replace_sources_item(&sources, 0, "fc2")
            }
        } else if sources.contains(&"mgstage") && (Regex::new(r"\d+\D+").unwrap().is_match(&file_number) || lo_file_number.contains("siro")) {
            replace_sources_item(&sources, 0, "mgstage")
        } else if sources.contains(&"gcolle") && Regex::new(r"\d{6}").unwrap().is_match(&file_number) {
            replace_sources_item(&sources, 0, "gcolle")
        }

        sources.into_iter().filter(|&s| self.adult_func_mapping.contains_key(s)).collect()
    }


    fn get_data_state(&self, data: &Value) -> bool {
        let titlle = data.get("title");
        let number = data.get("number");

        if titlle.is_none() || number.is_none(){
            return false;
        }
        let title  = titlle.unwrap().as_str().unwrap();
        if title.is_empty() || title == "null" {
            return false;
        }
        let number = number.unwrap().as_str().unwrap();
        if number.is_empty() || number == "null" {
            return false;
        }
        true
    }
}

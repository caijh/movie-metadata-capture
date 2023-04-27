use std::collections::HashMap;
use std::ops::Not;

use crate::azure_translator::AzureTranslator;
use regex::Regex;
use serde_json::Value;

use crate::config::{AppConfig, Parser, Translate};
use crate::parser::Movie;

#[derive(Default)]
pub struct Scraping {
    sources: Vec<String>,
    parsers: HashMap<String, Parser>,
    specified_source: Option<String>,
    debug: bool,
    translate: Translate,
}

fn replace_sources_item(sources: &mut Vec<&str>, index: usize, key: &str) {
    let _index = sources.iter().position(|s| s.to_owned() == key).unwrap();
    if _index > 0 {
        let ele = sources.remove(_index);
        sources.insert(index, ele);
    }
}

impl Scraping {
    pub fn new(config: &AppConfig) -> Self {
        let debug = config.debug_mode.switch;
        let sources: Vec<String> = config.sources.keys().cloned().collect();
        let parsers = config.sources.to_owned();
        let translate = config.translate.clone();
        Scraping {
            debug,
            sources,
            specified_source: None,
            parsers,
            translate,
        }
    }

    pub async fn search(
        &mut self,
        file_number: &str,
        sources: Option<String>,
        specified_source: Option<String>,
    ) -> Option<Movie> {
        self.specified_source = specified_source.map(|s| s);
        let sources = sources.unwrap_or_default();
        let sources: Vec<&str> = sources.split(",").filter(|s| s.is_empty().not()).collect();

        let movie = self.search_movie(file_number, sources).await;

        match movie {
            Some(mut movie) => {
                if self.translate.switch {
                    let json = serde_json::to_string(&movie).unwrap();
                    let json: Value = serde_json::from_str(json.as_str()).unwrap();
                    let translate_values: Vec<&str> = self.translate.values.split(",").collect();
                    for translate_value in translate_values {
                        if translate_value.is_empty() {
                            continue;
                        }
                        let text = json
                            .get(translate_value)
                            .unwrap()
                            .as_str()
                            .unwrap_or_default();
                        let t = if self.translate.engine == "azure" {
                            let translator = AzureTranslator {
                                service_url: self.translate.service_url.to_string(),
                                access_key: self.translate.access_key.to_string(),
                                region: self.translate.region.clone(),
                            };
                            translator.translate(text, "ja", "zh-Hans").await
                        } else {
                            None
                        };
                        if t.is_some() {
                            match translate_value {
                                "title" => movie.title = t.unwrap(),
                                "outline" => movie.outline = t.unwrap(),
                                _ => {}
                            };
                        }
                    }
                }
                Some(movie)
            }
            None => None,
        }
    }

    async fn search_movie(&mut self, file_number: &str, sources: Vec<&str>) -> Option<Movie> {
        let mut movie = None;

        let _sources: Vec<String> = if self.specified_source.is_some() {
            vec![self.specified_source.as_ref().unwrap().to_string()]
        } else {
            self.check_sources(sources, file_number)
        };
        if self.debug {
            println!("[+]sources {:?}", _sources);
        }
        for source in _sources {
            match self.parsers.get(source.as_str()) {
                Some(parser) => {
                    if self.debug {
                        println!("[+]select {}", source);
                    }
                    movie = parser.search(file_number).await;
                    if movie.is_some() {
                        if self.debug {
                            println!(
                                "[+]Find movie [{}] metadata on website '{}'",
                                file_number, source
                            );
                            println!("[+]Movie = {:?}", movie);
                        }
                        break;
                    } else {
                        movie = None;
                    }
                }
                None => continue,
            };
        }
        match movie {
            Some(m) => Some(m),
            None => {
                println!("[-]Movie Number [{}] not found!", file_number);
                None
            }
        }
    }

    fn check_sources(&self, sources: Vec<&str>, file_number: &str) -> Vec<String> {
        let mut _sources: Vec<&str> = if sources.is_empty() {
            self.sources.iter().map(|s| s.as_str()).collect()
        } else {
            sources
        };
        let lo_file_number = file_number.to_lowercase();
        if _sources.contains(&"carib")
            && Regex::new(r"^\d{6}-\d{3}").unwrap().is_match(&file_number)
        {
            replace_sources_item(&mut _sources, 0, "carib")
        } else if file_number.contains("item") || file_number.to_uppercase().contains("GETCHU") {
            replace_sources_item(&mut _sources, 0, "getchu")
        } else if lo_file_number.contains("rj")
            || lo_file_number.contains("vj")
            || Regex::new(r"[\u3040-\u309F\u30A0-\u30FF]+")
                .unwrap()
                .is_match(&file_number)
        {
            replace_sources_item(&mut _sources, 0, "getchu");
            replace_sources_item(&mut _sources, 1, "dlsite")
        } else if lo_file_number.contains("fc2") {
            if _sources.contains(&"fc2") {
                replace_sources_item(&mut _sources, 0, "fc2")
            }
        } else if _sources.contains(&"mgstage")
            && (Regex::new(r"\d+\D+").unwrap().is_match(&file_number)
                || lo_file_number.contains("siro"))
        {
            replace_sources_item(&mut _sources, 0, "mgstage")
        } else if _sources.contains(&"gcolle")
            && Regex::new(r"\d{6}").unwrap().is_match(&file_number)
        {
            replace_sources_item(&mut _sources, 0, "gcolle")
        }

        _sources
            .into_iter()
            .filter(|&s| self.parsers.contains_key(s))
            .map(|s| s.to_string())
            .collect()
    }
}

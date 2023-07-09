use std::collections::HashMap;
use std::ops::Not;

use crate::translator::AzureTranslator;

use serde_json::Value;

use crate::config::{AppConfig, NumberExtractor, Parser, Translate};
use crate::parser::Movie;

#[derive(Default)]
pub struct Scraping {
    sources: Vec<String>,
    parsers: HashMap<String, Parser>,
    specified_source: Option<String>,
    debug: bool,
    translate: Translate,
}

impl Scraping {
    pub fn new(config: &AppConfig) -> Self {
        let debug = config.debug_mode.switch;
        let sources: Vec<String> = config.get_sources().keys().cloned().collect();
        let parsers = config.get_sources().to_owned();
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
        number: &str,
        number_extractor: &NumberExtractor,
        sources: Option<String>,
        specified_source: Option<String>,
    ) -> Option<Movie> {
        self.specified_source = specified_source;
        let sources = sources.unwrap_or_default();
        let sources: Vec<&str> = sources.split(',').filter(|s| s.is_empty().not()).collect();

        let movie = self.search_movie(number, number_extractor, sources).await;

        match movie {
            Some(mut movie) => {
                movie = self.translate_movie(movie).await;
                Some(movie)
            }
            None => None,
        }
    }

    async fn translate_movie(&self, mut movie: Movie) -> Movie {
        if !self.translate.switch {
            return movie;
        }

        let json = serde_json::to_value(&movie).unwrap();
        let translate_values: Vec<&str> = self.translate.values.split(',').collect();

        for translate_value in translate_values {
            if translate_value.is_empty() {
                continue;
            }

            let text = json
                .get(translate_value)
                .and_then(Value::as_str)
                .unwrap_or_default();

            let t = match self.translate.engine.as_str() {
                "azure" => {
                    let translator = AzureTranslator {
                        service_url: self.translate.service_url.to_string(),
                        access_key: self.translate.access_key.to_string(),
                        region: self.translate.region.clone(),
                    };

                    translator.translate(text, "ja", "zh-Hans").await
                }
                _ => None,
            };

            if let Some(t) = t {
                match translate_value {
                    "title" => movie.title = t,
                    "outline" => movie.outline = t,
                    _ => {}
                }
            }
        }
        movie
    }

    async fn search_movie(
        &mut self,
        file_number: &str,
        number_extractor: &NumberExtractor,
        sources: Vec<&str>,
    ) -> Option<Movie> {
        let _sources: Vec<String> = if self.specified_source.is_some() {
            vec![self.specified_source.as_ref().unwrap().to_string()]
        } else {
            self.get_reorder_sources(sources, number_extractor)
        };
        if self.debug {
            println!("[+]Using sources {:?}", _sources);
        }

        let mut movie = None;
        for source in _sources {
            match self.parsers.get(source.as_str()) {
                Some(parser) => {
                    if self.debug {
                        println!("[+]Select source: {}", source);
                    }
                    movie = parser.search(file_number, self.debug).await;
                    if movie.is_some() {
                        if self.debug {
                            println!(
                                "[+]Find movie [{}] metadata on website '{}'",
                                file_number, source
                            );
                            println!("[+]Movie = {:?}", movie);
                        }
                        break;
                    }
                }
                None => continue,
            };
        }
        if movie.is_none() {
            println!("[-]Movie Number [{}] not found!", file_number);
        }
        movie
    }

    fn get_reorder_sources(
        &self,
        sources: Vec<&str>,
        number_extractor: &NumberExtractor,
    ) -> Vec<String> {
        let mut _sources: Vec<&str> = if sources.is_empty() {
            self.sources.iter().map(|s| s.as_str()).collect()
        } else {
            sources
        };

        let sources = number_extractor.sources.clone();
        if let Some(mut sources) = sources {
            sources.reverse();
            for ele in sources {
                replace_sources_item(&mut _sources, 0, &ele);
            }
        }

        _sources
            .into_iter()
            .filter(|&s| self.parsers.contains_key(s))
            .map(|s| s.to_string())
            .collect()
    }
}

// replace_sources_item() replaces an item in the source vector at the given index, with the respective key value. If the key is not found, nothing is done.
fn replace_sources_item(sources: &mut Vec<&str>, index: usize, key: &str) {
    let _index = sources.iter().position(|s| *s == key).unwrap();
    if _index > 0 {
        let ele = sources.remove(_index);
        sources.insert(index, ele);
    }
}

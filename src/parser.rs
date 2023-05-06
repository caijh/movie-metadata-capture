use regex::Regex;
use sxd_document::dom::Document;
use sxd_xpath::Value::Boolean;
use url::Url;

use crate::config::{Parser, StringFlow};
use crate::request::get_html_content;
use crate::xpath::{
    evaluate_xpath_node, value_to_string_use_handle, value_to_vec, value_to_vec_use_handle,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Movie {
    pub number: String,
    pub title: String,
    pub studio: String,
    pub year: String,
    pub outline: String,
    pub runtime: String,
    pub director: String,
    pub extra_fanart: Vec<String>,
    pub actor: Vec<(String, String)>,
    pub label: String,
    pub tag: Vec<String>,
    pub genre: String,
    pub premiered: String,
    pub release: String,
    pub cover: String,
    pub cover_small: String,
    pub trailer: String,
    pub website: String,
    pub series: String,
    pub uncensored: bool,
    pub userrating: String,
    pub max_userrating: String,
    pub uservotes: f64,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Actor {
    pub name: String,
}


impl Parser {
    pub async fn search(&self, number: &str, debug: bool) -> Option<Movie> {
        if let Some(age_check) = &self.age_check {
            let mut url = Url::parse(&age_check.url).unwrap();
            url.query_pairs_mut()
                .append_pair(&age_check.target_name, &age_check.target_url);
            let _ = get_html_content(url.as_str()).await;
        }

        let mut number = number.to_string();
        if let Some(site_search) = &self.site_search {
            let site_number = site_search.search(number.as_str()).await;
            if site_number.is_none() {
                return None;
            }
            number = if let Some(num) = site_number {
                num
            } else {
                number.to_owned()
            };
        }

        let detail_urls = &self.detail_url;
        for _url in detail_urls {
            let number_search = self
                .number_pre_handle
                .iter()
                .filter(|x| {
                    x.name == "*"
                        || number
                            .to_lowercase()
                            .contains(x.name.to_lowercase().as_str())
                })
                .last();

            let number = if let Some(number_search) = &number_search {
                let string_flow = StringFlow::new(&number_search.rule);
                let search_number = string_flow.process_string(number.as_str());
                search_number
            } else {
                number.to_owned()
            };

            let detail_url = _url.to_string() + number.as_str();
            let url = Url::parse(&detail_url).unwrap();
            if debug {
                println!("[+]{}", url);
            }
            if let Ok(html) = get_html_content(url.as_str()).await {
                let package = sxd_html::parse_html(html.as_str());
                let document = package.as_document();
                let movie = self.parse_to_movie(&document, detail_url);
                if self.is_movie_valid(&movie) {
                    return movie;
                }
            } else {
                println!("[-]Fail to get html content from {}", url);
            }
        }
        None
    }
    fn parse_to_movie(&self, document: &Document, detail_url: String) -> Option<Movie> {
        let mut number = evaluate_xpath_node(document.root(), self.expr_number.as_str())
            .unwrap()
            .string();
        if self.replace_number.is_some() {
            let string_flow = StringFlow::new(self.replace_number.as_ref().unwrap());
            number = string_flow.process_string(number.as_str());
        }

        let title = evaluate_xpath_node(document.root(), self.expr_title.as_str()).unwrap();
        let studio = evaluate_xpath_node(document.root(), self.expr_studio.as_str()).unwrap();
        let release = evaluate_xpath_node(document.root(), self.expr_release.as_str()).unwrap();
        let release = value_to_string_use_handle(release, &self.replace_release);
        let re = Regex::new(r"\d{4}").unwrap();
        let year = re.find(&release).map(|m| m.as_str().to_owned());
        let runtime = evaluate_xpath_node(document.root(), self.expr_runtime.as_str()).unwrap();
        let runtime = value_to_string_use_handle(runtime, &self.replace_runtime);
        let outline = evaluate_xpath_node(document.root(), self.expr_outline.as_str()).unwrap();
        let director = evaluate_xpath_node(document.root(), self.expr_director.as_str()).unwrap();
        let actor_name =
            evaluate_xpath_node(document.root(), self.expr_actor_name.as_str()).unwrap();
        let actor_name: Vec<String> = value_to_vec(actor_name);
        let actor_photo =
            evaluate_xpath_node(document.root(), self.expr_actor_photo.as_str()).unwrap();
        let actor_photo: Vec<String> =
            value_to_vec_use_handle(actor_photo, &self.replace_actor_photo);

        let mut actor = Vec::new();
        let mut iter1 = actor_name.into_iter();
        let mut iter2 = actor_photo.into_iter();
        loop {
            let elem1 = iter1.next();
            let elem2 = iter2.next();
            if elem1.is_none() {
                break;
            }
            let tuple = (elem1.unwrap_or_default(), elem2.unwrap_or_default());
            actor.push(tuple);
        }
        let expr_cover = self
            .expr_cover
            .replace("$cover_number", number.to_string().as_str());
        let mut cover = evaluate_xpath_node(document.root(), &expr_cover)
            .unwrap()
            .string();
        if self.replace_cover.is_some() {
            let string_flow = StringFlow::new(self.replace_cover.as_ref().unwrap());
            cover = string_flow.process_string(cover.as_str());
        }

        let expr_small_cover = self
            .expr_small_cover
            .replace("$cover_number", number.to_string().as_str());
        let cover_small = evaluate_xpath_node(document.root(), &expr_small_cover).unwrap();

        let extra_fanart =
            evaluate_xpath_node(document.root(), self.expr_extra_fanart.as_str()).unwrap();
        let extra_fanart: Vec<String> =
            value_to_vec_use_handle(extra_fanart, &self.replace_extra_fanart);
        let trailer = evaluate_xpath_node(document.root(), self.expr_trailer.as_str()).unwrap();
        let tags = evaluate_xpath_node(document.root(), self.expr_tags.as_str()).unwrap();
        let tags = value_to_vec_use_handle(tags, &self.replace_tags);
        let label = evaluate_xpath_node(document.root(), self.expr_label.as_str()).unwrap();
        let series = evaluate_xpath_node(document.root(), self.expr_series.as_str()).unwrap();
        let userrating =
            evaluate_xpath_node(document.root(), self.expr_userrating.as_str()).unwrap();
        let userrating = value_to_string_use_handle(userrating, &self.replace_userrating);
        let uservotes = evaluate_xpath_node(document.root(), self.expr_uservotes.as_str()).unwrap();
        let max_userrating = self.max_userrating.unwrap_or_default();
        let uncensored =
            evaluate_xpath_node(document.root(), self.expr_uncensored.as_str()).unwrap();
        let uncensored = match uncensored {
            Boolean(b) => b,
            _ => {
                if tags.contains(&"無码".to_string())
                    || tags.contains(&"無修正".to_string())
                    || tags.contains(&"uncensored".to_string())
                {
                    true
                } else {
                    false
                }
            }
        };
        Some(Movie {
            number: number.to_string(),
            title: title.string(),
            series: series.string(),
            studio: studio.string(),
            year: year.unwrap_or_default(),
            outline: outline.string().replace("\n", "").trim().to_string(),
            runtime,
            director: director.string(),
            extra_fanart,
            actor,
            label: label.string(),
            tag: tags,
            genre: "".to_string(),
            premiered: "".to_string(),
            release,
            cover,
            cover_small: cover_small.string(),
            trailer: trailer.string(),
            website: detail_url,
            uncensored,
            userrating,
            max_userrating: max_userrating,
            uservotes: uservotes.number(),
        })
    }

    fn is_movie_valid(&self, movie: &Option<Movie>) -> bool {
        if movie.is_none() {
            return false;
        }
        let movie = movie.as_ref().unwrap();
        let title = &movie.title;
        let number = &movie.number;

        if title.is_empty() || number.is_empty() {
            return false;
        }
        true
    }
}

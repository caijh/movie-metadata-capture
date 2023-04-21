use crate::config::Parser;
use crate::request::get_html_content;
use futures::FutureExt;
use regex::Regex;
use sxd_document::dom::Document;
use sxd_document::parser;
use sxd_xpath::evaluate_xpath;
use sxd_xpath::Value::Boolean;
use url::Url;

pub struct Movie {
    pub number: String,
    pub title: String,
    pub originaltitle: String,
    pub sorttitle: String,
    pub customrating: String,
    pub mpaa: String,
    pub set: String,
    pub studio: String,
    pub year: String,
    pub outline: String,
    pub plot: String,
    pub runtime: String,
    pub director: String,
    pub poster: String,
    pub thumb: String,
    pub fanart: String,
    pub extrafanart: Vec<String>,
    pub actors: Vec<Actor>,
    pub maker: String,
    pub label: String,
    pub tag: Vec<String>,
    pub genre: String,
    pub num: String,
    pub premiered: String,
    pub releasedate: String,
    pub release: String,
    pub rating: String,
    pub criticrating: String,
    pub ratings: Vec<Rating>,
    pub cover: String,
    pub cover_small: String,
    pub trailer: String,
    pub website: String,
    pub series: String,
    pub uncensored: bool,
}

pub struct Actor {
    pub name: String,
    pub thumb: String,
}

pub struct Rating {
    pub value: String,
    pub votes: String,
}

impl Parser {
    pub async fn search(&self, number: &str) -> Option<Movie> {
        let detail_urls = &self.detail_url;
        for url in detail_urls {
            let detail_url = url.to_string() + number;
            let url = if detail_url.contains("?") {
                Url::parse_with_params(&detail_url, &[("", "")]).unwrap()
            } else {
                Url::parse(&detail_url).unwrap()
            };

            if let Ok(html) = get_html_content(url.as_str()).await {
                let package = parser::parse(html.as_str());
                if let Ok(p) = package {
                    let document = p.as_document();
                    return self.parse_to_movie(&document, detail_url);
                }
            }
        }
        None
    }
    fn parse_to_movie(&self, document: &Document, detail_url: String) -> Option<Movie> {
        let number = evaluate_xpath(document, self.expr_number.as_str()).unwrap();
        let title = evaluate_xpath(document, self.expr_title.as_str()).unwrap();
        let studio = evaluate_xpath(&document, self.expr_studio.as_str()).unwrap();
        let release = evaluate_xpath(&document, self.expr_release.as_str()).unwrap();
        let release = release.string();
        let re = Regex::new(r"\d{4}").unwrap();
        let year = re.find(&release).map(|m| m.as_str().to_owned());
        let runtime = evaluate_xpath(&document, self.expr_runtime.as_str()).unwrap();
        let outline = evaluate_xpath(&document, self.expr_outline.as_str()).unwrap();
        let director = evaluate_xpath(&document, self.expr_director.as_str()).unwrap();
        let actors = evaluate_xpath(&document, self.expr_actor.as_str()).unwrap();
        let actors: Vec<Actor> = match actors {
            sxd_xpath::Value::Nodeset(nodes) => nodes
                .iter()
                .map(|node| Actor {
                    name: "".to_string(),
                    thumb: "".to_string(),
                })
                .collect(),
            _ => Vec::new(),
        };
        let cover = evaluate_xpath(&document, self.expr_cover.as_str()).unwrap();
        let cover_small = evaluate_xpath(&document, self.expr_smallcover.as_str()).unwrap();

        let extra_fanart = evaluate_xpath(&document, self.expr_extrafanart.as_str()).unwrap();
        let extra_fanart: Vec<String> = match extra_fanart {
            sxd_xpath::Value::Nodeset(nodes) => {
                nodes.iter().map(|node| node.string_value()).collect()
            }
            _ => Vec::new(),
        };
        let trailer = evaluate_xpath(&document, self.expr_trailer.as_str()).unwrap();
        let tags = evaluate_xpath(&document, self.expr_tags.as_str()).unwrap();
        let tags = match tags {
            sxd_xpath::Value::Nodeset(nodes) => {
                nodes.iter().map(|node| node.string_value()).collect()
            }
            _ => Vec::new(),
        };
        let label = evaluate_xpath(&document, self.expr_label.as_str()).unwrap();
        let series = evaluate_xpath(&document, self.expr_series.as_str()).unwrap();
        let userrating = evaluate_xpath(&document, self.expr_userrating.as_str()).unwrap();
        // let uservotes = evaluate_xpath(&document, self.expr_uservotes).unwrap_or_default();
        let uncensored = evaluate_xpath(&document, self.expr_uncensored.as_str()).unwrap();
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
            number: number.string(),
            title: title.string(),
            originaltitle: "".to_string(),
            sorttitle: "".to_string(),
            customrating: "".to_string(),
            mpaa: "".to_string(),
            set: "".to_string(),
            series: series.string(),
            studio: studio.string(),
            year: year.unwrap_or_default(),
            outline: outline.string(),
            plot: "".to_string(),
            runtime: runtime.string(),
            director: director.string(),
            poster: "".to_string(),
            thumb: "".to_string(),
            fanart: "".to_string(),
            extrafanart: extra_fanart,
            actors,
            maker: "".to_string(),
            label: label.string(),
            tag: tags,
            genre: "".to_string(),
            num: "".to_string(),
            premiered: "".to_string(),
            releasedate: "".to_string(),
            release,
            rating: userrating.string(),
            criticrating: "".to_string(),
            ratings: Vec::new(),
            cover: cover.string(),
            cover_small: cover_small.string(),
            trailer: trailer.string(),
            website: detail_url,
            uncensored,
        })
    }
}

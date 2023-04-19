use clap::builder::Str;
use regex::Regex;
use sxd_document::dom::Document;
use sxd_document::parser;
use sxd_xpath::evaluate_xpath;
use sxd_xpath::Value::Boolean;
use crate::config::{Parser, Source};
use crate::files::read_file_line_and_remove_repeat_line;
use crate::movie::{Actor, Movie};
use crate::request::get_html_content;


impl Parser {

    pub fn search(&self, number: &str) -> Option<Movie> {
        let detail_url = if self.special_url.is_empty() {
            self.source.detail_url.to_string() + number
        } else {
            self.special_url
        };
        let html = get_html_content(&detail_url)?;
        let package = parser::parse(html).expect("failed to parse XML");
        let document = package.as_document();
        self.parse_to_movie(&document, detail_url)
    }

    fn parse_to_movie(&self, document: &Document, detail_url: String) -> Option<Movie> {
        let number = evaluate_xpath(document, self.expr_number).unwrap_or_default();
        let title = evaluate_xpath(document, self.expr_title).unwrap_or_default();
        let studio = evaluate_xpath(&document, self.expr_studio).unwrap_or_default();
        let release = evaluate_xpath(&document, self.expr_release).unwrap_or_default();
        let release = release.string();
        let re = Regex::new(r"\d{4}").unwrap();
        let year = re.find(&release).map(|m| m.as_str().to_owned());
        let runtime = evaluate_xpath(&document, self.expr_runtime).unwrap_or_default();
        let outline = evaluate_xpath(&document, self.expr_outline).unwrap_or_default();
        let director = evaluate_xpath(&document, self.expr_director).unwrap_or_default();
        let actors = evaluate_xpath(&document, self.expr_actor).unwrap_or_default();
        let actors: Vec<Actor> = match actors
        {
            sxd_xpath::Value::Nodeset(nodes) =>
                {
                    nodes
                        .iter()
                        .map(|node| Actor { name: "".to_string(), thumb: "".to_string() })
                        .collect()
                }
            _ => Vec::new(),
        };
        let cover = evaluate_xpath(&document, self.expr_cover).unwrap_or_default();
        let cover_small = evaluate_xpath(&document, self.expr_smallcover).unwrap_or_default();

        let extra_fanart = evaluate_xpath(&document, self.expr_extrafanart).unwrap_or_default();
        let extra_fanart: Vec<String> = match extra_fanart
        {
            sxd_xpath::Value::Nodeset(nodes) =>
                {
                    nodes
                        .iter()
                        .map(|node| node.text().to_string())
                        .collect()
                }
            _ => Vec::new(),
        };
        let trailer = evaluate_xpath(&document, self.expr_trailer).unwrap_or_default();
        let tags = evaluate_xpath(&document, self.expr_tags).unwrap_or_default();
        let tags = match tags
        {
            sxd_xpath::Value::Nodeset(nodes) =>
                {
                    nodes
                        .iter()
                        .map(|node| node.text().to_string().trim())
                        .collect()
                }
            _ => Vec::new(),
        };
        let label = evaluate_xpath(&document, self.expr_label).unwrap_or_default();
        let series = evaluate_xpath(&document, self.expr_series).unwrap_or_default();
        // let userrating = evaluate_xpath(&document, self.expr_userrating).unwrap_or_default();
        // let uservotes = evaluate_xpath(&document, self.expr_uservotes).unwrap_or_default();
        let uncensored = evaluate_xpath(&document, self.expr_uncensored).unwrap_or_default();
        let uncensored = match uncensored {
            Boolean(b) => b,

            _ => {
                if tags.contains("無码".as_ref()) || tags.contains("無修正".as_ref()) || tags.contains("uncensored".as_ref()) {
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

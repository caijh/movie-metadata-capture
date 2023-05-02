use serde::{Deserialize, Serialize};
use sxd_document::dom::Document;
use url::Url;

use crate::config::{Rule, StringFlow};
use crate::request::get_html_content;
use crate::xpath::evaluate_xpath_node;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SiteSearch {
    pub url: String,
    pub number_pre_handle: Option<Vec<Rule>>,
    pub expr_number: String,
    pub expr_id: String,
    pub number_post_handle: Option<Vec<Rule>>,
    pub id_post_handle: Option<Vec<Rule>>,
}

impl SiteSearch {
    pub async fn search(&self, number: &str) -> String {
        let search_number = if let Some(number_pre_handle) = &self.number_pre_handle {
            let string_flow = StringFlow::new(number_pre_handle);
            let num = string_flow.process_string(number);
            num
        } else {
            number.to_owned()
        };
        let search_url = self.url.to_string() + search_number.as_str();
        let url = Url::parse(&search_url).unwrap();
        let mut number_ids = Vec::new();
        if let Ok(html) = get_html_content(url.as_str()).await {
            let package = sxd_html::parse_html(html.as_str());
            let document = package.as_document();
            number_ids = self.parse_search_result(&document);
        }
        let matching_id = number_ids.iter()
            .filter(|(num, _id)| num == number)
            .map(|(_num, id)| id.to_owned())
            .last();
        match matching_id {
            Some(id) => id,
            None => "".to_owned(),
        }
    }

    fn parse_search_result(&self, document: &Document) -> Vec<(String, String)> {
        let numbers =
            evaluate_xpath_node(document.root(), self.expr_number.as_str()).unwrap();
        let numbers: Vec<String> = match numbers {
            sxd_xpath::Value::Nodeset(nodes) => {
                if self.number_post_handle.is_some() {
                    let string_flow = StringFlow::new(self.number_post_handle.as_ref().unwrap());
                    nodes
                        .iter()
                        .map(|node| string_flow.process_string(node.string_value().as_str()))
                        .collect()
                } else {
                    nodes.iter().map(|node| node.string_value()).collect()
                }
            }
            _ => Vec::new(),
        };
        let ids =
            evaluate_xpath_node(document.root(), self.expr_id.as_str()).unwrap();
        let ids: Vec<String> = match ids {
            sxd_xpath::Value::Nodeset(nodes) => {
                if self.id_post_handle.is_some() {
                    let string_flow = StringFlow::new(self.id_post_handle.as_ref().unwrap());
                    nodes
                        .iter()
                        .map(|node| string_flow.process_string(node.string_value().as_str()))
                        .collect()
                } else {
                    nodes.iter().map(|node| node.string_value()).collect()
                }
            }
            _ => Vec::new(),
        };
        let mut number_ids = Vec::new();
        let mut iter1 = numbers.into_iter();
        let mut iter2 = ids.into_iter();
        loop {
            let elem1 = iter1.next();
            let elem2 = iter2.next();
            if elem1.is_none() {
                break;
            }
            let tuple = (elem1.unwrap_or_default(), elem2.unwrap_or_default());
            number_ids.push(tuple);
        }
        number_ids
    }
}

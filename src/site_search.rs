

use serde::{Deserialize, Serialize};
use sxd_document::dom::Document;
use url::Url;

use crate::config::{Rule, StringFlow};
use crate::request::get_html_content;
use crate::xpath::{evaluate_xpath_node, value_to_vec_use_handle};

// SiteSearch stores the url to search for IDs and Numbers, options for pre-processing the numbers, the
// expressions for the numbers and IDs, and options for post-processing the numbers and IDs.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SiteSearch {
    pub url: String,
    pub site_number_pre_handle: Option<Vec<Rule>>,
    pub expr_number: String,
    pub expr_id: String,
    pub site_number_post_handle: Option<Vec<Rule>>,
    pub site_id_post_handle: Option<Vec<Rule>>,
}

impl SiteSearch {
    /// Searches for the provided number and returns its id as a String if found.
    ///
    /// # Parameters
    /// -  `number` : The number to search
    ///
    /// # Returns
    /// -  `Option<String>` : An option containing the number's id as a String if found,  `None`  otherwise
    pub async fn search(&self, number: &str) -> Option<String> {
        let search_number = if let Some(number_pre_handle) = &self.site_number_pre_handle {
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

        for x in number_ids {
            if x.0.as_str() == number {
                return Some(x.1);
            }
        }
        None
    }

    fn parse_search_result(&self, document: &Document) -> Vec<(String, String)> {
        let numbers = evaluate_xpath_node(document.root(), self.expr_number.as_str()).unwrap();
        let numbers = value_to_vec_use_handle(numbers, &self.site_number_post_handle);
        let ids = evaluate_xpath_node(document.root(), self.expr_id.as_str()).unwrap();
        let ids = value_to_vec_use_handle(ids, &self.site_id_post_handle);
        let mut number_ids = Vec::new();
        for i in 0..numbers.len() {
            number_ids.push((
                numbers.iter().nth(i).unwrap().to_owned(),
                ids.iter().nth(i).unwrap().to_owned(),
            ));
        }
        number_ids
    }
}

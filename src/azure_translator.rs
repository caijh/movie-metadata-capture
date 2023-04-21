use serde_derive::{Deserialize, Serialize};

use crate::request::client;

pub struct AzureTranslator {
    service_url: String,
    access_key: String,
    region: Option<String>,
}

#[derive(Debug, Serialize)]
struct AzureRequest<'a> {
    text: &'a str,
}

#[derive(Debug, Deserialize)]
struct AzureResponse {
    translations: Vec<Translation>,
}

#[derive(Debug, Deserialize)]
struct Translation {
    text: String,
}


impl AzureTranslator {
    pub fn new(service_url: String, access_key: String, region: Option<String>) -> AzureTranslator {
        AzureTranslator {
            service_url,
            access_key,
            region,
        }
    }

    pub async fn translate(&self, text: &str, from_lang: &str, to_lang: &str) -> Option<String> {
        let request_body = [AzureRequest { text }];
        let request_body = serde_json::to_string(&request_body).unwrap();
        let params = [("from", from_lang), ("to", to_lang)];
        let client = client().await.expect("fail to get client");
        let region = &self.region;
        let region = if region.is_some() {
            region.as_ref().unwrap().to_string()
        } else {
            "global".to_string()
        };
        let response = client
            .post(&self.service_url)
            .header("Content-Type", "application/json")
            .header("Ocp-Apim-Subscription-Key", &self.access_key)
            .header("Ocp-Apim-Subscription-Region", region)
            .query(&params)
            .body(request_body)
            .send().await.unwrap();
        match response.text().await {
            Ok(text) => {
                let json_data: Vec<AzureResponse> = serde_json::from_str(&text).unwrap();
                let translation = &json_data[0].translations[0].text;
                Some(translation.to_string())
            }
            Err(_) => {
                None
            }
        }
    }
}

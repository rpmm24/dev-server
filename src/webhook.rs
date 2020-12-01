use serde::Serialize;

use super::Result;

#[derive(Serialize)]
pub struct Payload {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

#[derive(Clone)]
pub struct Client {
    url: String,
    reqwest: reqwest::Client,
}

impl Client {
    pub fn open(url: impl Into<String>) -> Client {
        Client {
            url: url.into(),
            reqwest: reqwest::Client::new(),
        }
    }

    pub async fn post(&self, payload: &Payload) -> Result<()> {
        self.reqwest.post(&self.url)
            .json(payload)
            .send().await?;
        Ok(())
    }
}

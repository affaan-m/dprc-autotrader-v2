use anyhow::Result;
use reqwest::{Client, header};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct TwitterClient {
    client: Client,
    username: String,
    cookies: String,
    last_tweet_time: Arc<Mutex<i64>>,
}

impl TwitterClient {
    pub fn new(username: String, cookies: String) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::COOKIE,
            header::HeaderValue::from_str(&cookies)?,
        );

        let client = Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self {
            client,
            username,
            cookies,
            last_tweet_time: Arc::new(Mutex::new(0)),
        })
    }

    pub async fn post_tweet(&self, text: &str) -> Result<String> {
        let json = serde_json::json!({
            "text": text,
        });

        let response = self.client
            .post("https://api.twitter.com/2/tweets")
            .json(&json)
            .send()
            .await?
            .json::<Value>()
            .await?;

        Ok(response["data"]["id"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Failed to get tweet ID"))?
            .to_string())
    }

    pub async fn reply_to_tweet(&self, reply_to_id: &str, text: &str) -> Result<String> {
        let json = serde_json::json!({
            "text": text,
            "reply": {
                "in_reply_to_tweet_id": reply_to_id
            }
        });

        let response = self.client
            .post("https://api.twitter.com/2/tweets")
            .json(&json)
            .send()
            .await?
            .json::<Value>()
            .await?;

        Ok(response["data"]["id"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Failed to get tweet ID"))?
            .to_string())
    }
} 
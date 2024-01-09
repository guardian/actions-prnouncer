use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub struct GoogleChatMessage {
    text: String,
}

impl GoogleChatMessage {
    pub fn from(text: String) -> GoogleChatMessage {
        GoogleChatMessage { text: text }
    }

    pub async fn send(
        self,
        webhook_url: &String,
        thread_key: &String,
    ) -> Result<GoogleChatMessage> {
        let webhook_url = Url::parse_with_params(webhook_url,
        &[("messageReplyOption", "REPLY_MESSAGE_FALLBACK_TO_NEW_THREAD")])?;

        let response = reqwest::Client::new()
            .post(webhook_url.as_str().replace("{threadKey}", thread_key))
            .body(serde_json::to_string(&self)?)
            .header("User-Agent", "GU-PR-Bot")
            .send()
            .await?
            .text()
            .await
            .context(format!("Failed to get response from: {}", &webhook_url))?;

        serde_json::from_str(&response).context(format!(
            "Failed to parse JSON when querying {}: {}",
            &webhook_url, response
        ))
    }
}

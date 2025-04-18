use anyhow::{Context, Result};
use log::info;
use serde::{Deserialize, Serialize};
use std::time;
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub struct GoogleChatMessage {
    text: String,
}

impl GoogleChatMessage {
    pub fn from(text: String) -> GoogleChatMessage {
        GoogleChatMessage { text }
    }

    /// Build a webhook URL with threadKey set to the provided value, and messageReplyOption set to allow threading.
    pub fn build_webhook_url(webhook_url: &str, thread_key: &str) -> Result<String> {
        let url = Url::parse(webhook_url)?;
        // strip any existing threadKey / messageReplyOption parameters
        let other_params = url
            .query_pairs()
            .filter(|(name, _)| name.ne("threadKey") && name.ne("messageReplyOption"));
        // rebuild URL with the provided threadKey value, and our desired messageReplyOption
        let mut updated_url = url.clone();
        updated_url
            .query_pairs_mut()
            .clear()
            .extend_pairs(other_params)
            .extend_pairs(&[
                ("messageReplyOption", "REPLY_MESSAGE_FALLBACK_TO_NEW_THREAD"),
                ("threadKey", thread_key),
            ]);
        Ok(updated_url.as_str().to_string())
    }

    pub async fn send(
        self,
        webhook_url: &str,
        thread_key: &str,
        attempt: i32,
    ) -> Result<GoogleChatMessage> {
        let max_attempts = 3;
        let url = GoogleChatMessage::build_webhook_url(webhook_url, thread_key)?;

        let response = reqwest::Client::new()
            .post(url.to_string())
            .body(serde_json::to_string(&self)?)
            .header("User-Agent", "GU-PR-Bot")
            .send()
            .await?;

        if response
            .status()
            .eq(&reqwest::StatusCode::TOO_MANY_REQUESTS)
            && attempt < max_attempts
        {
            /*
            Sleep for 1.5 seconds to avoid rate limiting, at 60 requests per minute.
            See https://developers.google.com/workspace/chat/limits
            */
            info!(
                "Received {} response. Sleeping for 1.5 seconds before retrying (attempt {})",
                response.status(),
                attempt
            );
            tokio::time::sleep(time::Duration::from_millis(1500)).await;
            Box::pin(self.send(webhook_url, thread_key, attempt + 1)).await
        } else {
            let response_text = response.text().await.context(format!(
                "Failed to get response from: {} after {} attempts",
                &webhook_url, attempt
            ))?;

            serde_json::from_str(&response_text).context(format!(
                "Failed to parse JSON when querying {}: {}",
                &webhook_url, response_text
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_threaded_webhook_with_placeholder() {
        let result = GoogleChatMessage::build_webhook_url(
            "https://example.com/ABCDEF?threadKey={threadKey}",
            "1234",
        );
        assert_eq!(Result::unwrap(result), String::from("https://example.com/ABCDEF?messageReplyOption=REPLY_MESSAGE_FALLBACK_TO_NEW_THREAD&threadKey=1234"))
    }

    #[test]
    fn test_build_threaded_webhook_without_placeholder() {
        let result = GoogleChatMessage::build_webhook_url("https://example.com/ABCDEF", "1234");
        assert_eq!(Result::unwrap(result), String::from("https://example.com/ABCDEF?messageReplyOption=REPLY_MESSAGE_FALLBACK_TO_NEW_THREAD&threadKey=1234"))
    }

    #[test]
    fn test_preserves_other_existing_params() {
        let result =
            GoogleChatMessage::build_webhook_url("https://example.com/ABCDEF?foo=bar", "1234");
        assert_eq!(Result::unwrap(result), String::from("https://example.com/ABCDEF?foo=bar&messageReplyOption=REPLY_MESSAGE_FALLBACK_TO_NEW_THREAD&threadKey=1234"))
    }

    #[test]
    fn test_overrides_existing_messagereplyoption_parameter() {
        let result = GoogleChatMessage::build_webhook_url(
            "https://example.com/ABCDEF?messageReplyOption=REPLY_MESSAGE",
            "1234",
        );
        assert_eq!(Result::unwrap(result), String::from("https://example.com/ABCDEF?messageReplyOption=REPLY_MESSAGE_FALLBACK_TO_NEW_THREAD&threadKey=1234"))
    }
}

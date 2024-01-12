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

    /// Build a webhook URL with threadKey set to the provided value, and messageReplyOption set to allow threading.
    pub fn build_webhook_url(webhook_url: &String, thread_key: &String) -> Result<String> {
        let url = Url::parse(webhook_url)?;
        // strip any existing threadKey / messageReplyOption parameters
        let other_params = url
            .query_pairs()
            .filter(|(name, _)| name.ne("threadKey") && name.ne("messageReplyOption"));
        // rebuild URL with the provided threadKey value, and our desired messageReplyOption
        let mut updated_url = url.clone();
        updated_url.query_pairs_mut()
            .clear()
            .extend_pairs(other_params)
            .extend_pairs(&[
                (&"messageReplyOption".to_string(), &"REPLY_MESSAGE_FALLBACK_TO_NEW_THREAD".to_string()),
                (&"threadKey".to_string(), thread_key)
            ]);
        Ok(updated_url.as_str().to_string())
    }

    pub async fn send(
        self,
        webhook_url: &String,
        thread_key: &String,
    ) -> Result<GoogleChatMessage> {
        let url = GoogleChatMessage::build_webhook_url(&webhook_url, &thread_key)?;

        let response = reqwest::Client::new()
            .post(url.to_string())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_threaded_webhook_with_placeholder() {
        let result = GoogleChatMessage::build_webhook_url(&String::from("https://example.com/ABCDEF?threadKey={threadKey}"), &String::from("1234"));
        assert_eq!(Result::unwrap(result), String::from("https://example.com/ABCDEF?messageReplyOption=REPLY_MESSAGE_FALLBACK_TO_NEW_THREAD&threadKey=1234"))
    }

    #[test]
    fn test_build_threaded_webhook_without_placeholder() {
        let result = GoogleChatMessage::build_webhook_url(&String::from("https://example.com/ABCDEF"), &String::from("1234"));
        assert_eq!(Result::unwrap(result), String::from("https://example.com/ABCDEF?messageReplyOption=REPLY_MESSAGE_FALLBACK_TO_NEW_THREAD&threadKey=1234"))
    }

    #[test]
    fn test_preserves_other_existing_params() {
        let result = GoogleChatMessage::build_webhook_url(&String::from("https://example.com/ABCDEF?foo=bar"), &String::from("1234"));
        assert_eq!(Result::unwrap(result), String::from("https://example.com/ABCDEF?foo=bar&messageReplyOption=REPLY_MESSAGE_FALLBACK_TO_NEW_THREAD&threadKey=1234"))
    }

    #[test]
    fn test_overrides_existing_messagereplyoption_parameter() {
        let result = GoogleChatMessage::build_webhook_url(&String::from("https://example.com/ABCDEF?messageReplyOption=REPLY_MESSAGE"), &String::from("1234"));
        assert_eq!(Result::unwrap(result), String::from("https://example.com/ABCDEF?messageReplyOption=REPLY_MESSAGE_FALLBACK_TO_NEW_THREAD&threadKey=1234"))
    }
}
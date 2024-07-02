use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use getset::Getters;
use serde::Deserialize;

#[derive(Deserialize, Getters, Debug)]
#[getset(get = "pub", get_copy = "pub")]
pub struct GithubPullRequest {
    id: usize,
    url: String,
    html_url: String,
    title: String,
    user: GithubUser,
    draft: bool,
    number: usize,
    head: GithubBranch,
    labels: Vec<GithubLabel>,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize, Getters, Debug)]
#[getset(get = "pub", get_copy = "pub")]
pub struct GithubBranch {
    repo: GithubRepository,
}

#[derive(Deserialize, Getters, Debug)]
#[getset(get = "pub", get_copy = "pub")]
pub struct GithubRepository {
    name: String,
}

#[derive(Deserialize, Getters, Debug)]
#[getset(get = "pub", get_copy = "pub")]
pub struct GithubLabel {
    name: String,
}

#[derive(Deserialize, Getters, Debug)]
#[getset(get = "pub")]
pub struct GithubUser {
    id: usize,
    login: String,
}

#[derive(Deserialize, Getters, Debug)]
#[getset(get = "pub")]
pub struct GithubReview {
    id: usize,
    state: String,
}

impl GithubPullRequest {
    pub async fn list(repository: &str, token: &str) -> Result<Vec<GithubPullRequest>> {
        let api_url = format!(
            "https://api.github.com/repos/{}/pulls?state=open&per_page=100",
            repository
        );

        let response = reqwest::Client::new()
            .get(&api_url)
            .header("User-Agent", "GU-PR-Bot")
            .bearer_auth(token)
            .send()
            .await?
            .text()
            .await
            .context(format!("Failed to get response from: {}", &api_url))?;

        serde_json::from_str(&response).context(format!(
            "Failed to parse JSON when querying {}: {}",
            &api_url, response
        ))
    }

    pub async fn reviews(&self, token: &str) -> Result<Vec<GithubReview>> {
        let api_url = format!("{}/reviews", self.url());

        let response = reqwest::Client::new()
            .get(&api_url)
            .header("User-Agent", "GU-PR-Bot")
            .bearer_auth(token)
            .send()
            .await?
            .text()
            .await
            .context(format!("Failed to get response from: {}", &api_url))?;

        serde_json::from_str(&response).context(format!(
            "Failed to parse JSON when querying {}: {}",
            &api_url, response
        ))
    }
}

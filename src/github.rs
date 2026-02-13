use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct GithubPullRequest {
    pub id: usize,
    pub url: String,
    pub html_url: String,
    pub title: String,
    pub user: GithubUser,
    pub draft: bool,
    pub number: usize,
    pub head: GithubBranch,
    pub labels: Vec<GithubLabel>,
    pub created_at: DateTime<Utc>,
    pub additions: u32,
    pub deletions: u32,
}

#[derive(Deserialize, Debug)]
pub struct GithubBranch {
    pub repo: GithubRepository,
}

#[derive(Deserialize, Debug)]
pub struct GithubRepository {
    pub name: String,
    pub visibility: String,
}

#[derive(Deserialize, Debug)]
pub struct GithubLabel {
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct GithubUser {
    pub id: usize,
    pub login: String,
    pub r#type: String,
}

#[derive(Deserialize, Debug)]
pub struct GithubReview {
    pub id: usize,
    pub state: String,
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
        let api_url = format!("{}/reviews", self.url);

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

impl GithubUser {
    pub async fn list(team_id: &str, token: &str) -> Result<Vec<GithubUser>> {
        let api_url = format!(
            "https://api.github.com/orgs/guardian/teams/{}/members",
            team_id
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
}

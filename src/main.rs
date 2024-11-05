mod github;
mod google;

use anyhow::{Context, Error};
use chrono::{DateTime, Datelike, Utc};
use log::{info, Level};
use std::env;

use github::GithubPullRequest;
use google::GoogleChatMessage;

const PUBLIC_REPO: &str = "public";

async fn scan_repository(
    repository_name: &str,
    github_token: &str,
    ignored_users: &[&str],
    announced_users: &Option<Vec<usize>>,
    ignored_labels: &[&str],
) -> Result<Vec<GithubPullRequest>, Error> {
    info!("\nStarting PR scan of {}", repository_name);

    let pull_requests = GithubPullRequest::list(repository_name, github_token).await?;
    let mut pull_requests_to_review: Vec<GithubPullRequest> = vec![];

    info!("Found {} PR's", pull_requests.len());

    for pull_request in pull_requests {
        let is_public = pull_request.head.repo.visibility == PUBLIC_REPO;

        if is_public {
            info!("Processing PR {}({})", pull_request.id, pull_request.title);
        }

        if pull_request.draft {
            if is_public {
                info!(
                    "Ignoring PR {}({}) as it is a draft",
                    pull_request.id, pull_request.title
                );
            }
            continue;
        }

        if ignored_users.contains(&pull_request.user.id.to_string().as_str()) {
            if is_public {
                info!(
                    "Ignoring PR {}({}) as it was raised by an ignored user {}({})",
                    pull_request.id,
                    pull_request.title,
                    pull_request.user.id,
                    pull_request.user.login
                );
            }
            continue;
        }

        if let Some(announced_users) = announced_users {
            if !announced_users.contains(&pull_request.user.id) {
                if is_public {
                    info!("Users to announce: {:?}", announced_users);
                    info!(
                    "Ignoring PR {}({}) as it was raised by a user not included in the announced users list {}({})",
                    pull_request.id,
                    pull_request.title,
                    pull_request.user.id,
                    pull_request.user.login
                );
                }
                continue;
            }
        }

        let mut has_ignore_label = false;

        for label in &pull_request.labels {
            if ignored_labels.contains(&label.name.as_str()) {
                if is_public {
                    info!(
                        "Ignoring PR {}({}) as it has an ignored label ({})",
                        pull_request.id, pull_request.title, label.name
                    );
                }
                has_ignore_label = true;
            }
        }

        if has_ignore_label {
            continue;
        }

        let pull_request_reviews = pull_request.reviews(github_token).await?;

        if is_public {
            info!(
                "Found {} reviews for PR {}({})",
                pull_request_reviews.len(),
                pull_request.id,
                pull_request.title
            );
        }

        let mut has_approved_reviews = false;

        for pull_request_review in pull_request_reviews {
            if is_public {
                info!(
                    "Processing review {} for PR {}({})",
                    pull_request_review.id, pull_request.id, pull_request.title
                );
            }

            if pull_request_review.state == "APPROVED" {
                has_approved_reviews = true;
            }
        }

        if !has_approved_reviews {
            pull_requests_to_review.push(pull_request);
        }
    }

    Ok(pull_requests_to_review)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    simple_logger::init_with_level(Level::Info)?;

    let repositories: String =
        env::var("GITHUB_REPOSITORIES").context("GITHUB_REPOSITORIES must be set")?;
    let repositories: Vec<&str> = repositories.split(',').collect();
    let github_token: String = env::var("GITHUB_TOKEN").context("GITHUB_TOKEN must be set")?;
    let webhook_url: String =
        env::var("GOOGLE_WEBHOOK_URL").context("GOOGLE_WEBHOOK_URL must be set")?;
    let ignored_users: String = env::var("GITHUB_IGNORED_USERS").unwrap_or("".to_string());
    let ignored_users: Vec<&str> = ignored_users.split(',').collect();
    let announced_users: Option<Vec<usize>> =
        env::var("GITHUB_ANNOUNCED_USERS").ok().and_then(|s| {
            if s.is_empty() {
                None
            } else {
                Some(s.split(',').map(|id| id.parse().unwrap()).collect())
            }
        });
    let ignored_labels: String = env::var("GITHUB_IGNORED_LABELS").unwrap_or("".to_string());
    let ignored_labels: Vec<&str> = ignored_labels.split(',').collect();
    let show_pr_age: bool = env::var("SHOW_PR_AGE")
        .map(|v| v == "true")
        .unwrap_or(false);

    let mut pull_requests_to_review: Vec<GithubPullRequest> = vec![];

    for repository in repositories {
        pull_requests_to_review.append(
            &mut scan_repository(
                repository,
                &github_token,
                &ignored_users,
                &announced_users,
                &ignored_labels,
            )
            .await?,
        );
    }

    if !pull_requests_to_review.is_empty() {
        let weekday = match chrono::offset::Local::now().date_naive().weekday() {
            chrono::Weekday::Mon => "Monday",
            chrono::Weekday::Tue => "Tuesday",
            chrono::Weekday::Wed => "Wednesday",
            chrono::Weekday::Thu => "Thursday",
            chrono::Weekday::Fri => "Friday",
            chrono::Weekday::Sat => "Saturday",
            chrono::Weekday::Sun => "Sunday",
        };

        let mut message = String::new();

        message.push_str(format!("🧵 {} Reviews 🧵\n", weekday).as_str());
        message.push_str("(PR's can be hidden from this bot by adding the Stale tag)\n");
        message.push_str("--------------------\n");
        message.push_str("This message is brought to you by <github.com/guardian/actions-prnouncer|guardian/actions-prnouncer>, ");
        message.push_str("with configuration from <github.com/guardian/prnouncer-config|guardian/prnouncer-config>\n");
        message.push_str("--------------------\n\n");

        let thread_key = format!("pr-thread-{}", chrono::offset::Local::now());

        info!("Using thread key {}", thread_key);

        GoogleChatMessage::from(message)
            .send(&webhook_url, &thread_key)
            .await?;

        for pull_request in pull_requests_to_review {
            info!(
                "Sending message for PR {} #{}",
                pull_request.head.repo.name, pull_request.number
            );
            GoogleChatMessage::from(make_message(pull_request, show_pr_age))
                .send(&webhook_url, &thread_key)
                .await?;
        }
    } else {
        info!("No open PRs found, no action taken.");
    }

    Ok(())
}

fn make_message(pull_request: GithubPullRequest, show_pr_age: bool) -> String {
    let message = format!(
        "<{}|{}#{}> - {}",
        pull_request.html_url.replace("https://", ""),
        pull_request.head.repo.name,
        pull_request.number,
        pull_request.title
    );

    let age_output = if show_pr_age {
        format!(" - (_{}_)", get_age(Utc::now(), pull_request.created_at))
    } else {
        "".to_string()
    };

    let user = if pull_request.user.r#type.to_lowercase() == "bot" {
        format!("🤖 {}", pull_request.user.login)
    } else {
        format!("👤 {}", pull_request.user.login)
    };

    format!("{}{} \n\n{}\n", message, age_output, user)
}

fn get_age(d1: DateTime<Utc>, d2: DateTime<Utc>) -> String {
    match d1.signed_duration_since(d2).num_days() {
        0 => "NEW".to_string(),
        1 => "opened 1 day ago".to_string(),
        n => format!("opened {} days ago", n),
    }
}

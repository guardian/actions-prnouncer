extern crate getset;

mod github;
mod google;

use anyhow::{Context, Error};
use chrono::Datelike;
use log::{info, Level};
use std::env;

use github::GithubPullRequest;
use google::GoogleChatMessage;

async fn scan_repository(
    repository_name: String,
    github_token: &String,
    ignored_users: &Vec<&str>,
    ignored_labels: &Vec<&str>,
) -> Result<Vec<GithubPullRequest>, Error> {
    info!("Starting PR scan of {}", repository_name);

    let pull_requests = GithubPullRequest::list(repository_name, &github_token).await?;
    let mut pull_requests_to_review: Vec<GithubPullRequest> = vec![];

    info!("Found {} PR's", pull_requests.len());

    for pull_request in pull_requests {
        info!(
            "Processing PR {}({})",
            pull_request.id(),
            pull_request.title()
        );

        if *pull_request.draft() {
            info!(
                "Ignoring PR {}({}) as it is a draft",
                pull_request.id(),
                pull_request.title()
            );
            continue;
        }

        if ignored_users.contains(&pull_request.user().id().to_string().as_str()) {
            info!(
                "Ignoring PR {}({}) as it was raised by an ignored user {}({})",
                pull_request.id(),
                pull_request.title(),
                pull_request.user().id(),
                pull_request.user().login()
            );
            continue;
        }

        let mut has_ignore_label = false;

        for label in pull_request.labels() {
            if ignored_labels.contains(&label.name().as_str()) {
                info!(
                    "Ignoring PR {}({}) as it has an ignored label ({})",
                    pull_request.id(),
                    pull_request.title(),
                    label.name()
                );
                has_ignore_label = true;
            }
        }

        if has_ignore_label {
            continue;
        }

        let pull_request_reviews = pull_request.reviews(&github_token).await?;

        info!(
            "Found {} reviews for PR {}({})",
            pull_request_reviews.len(),
            pull_request.id(),
            pull_request.title()
        );

        let mut has_approved_reviews = false;
        let has_reviews_with_outstanding_comments = false;

        for pull_request_review in pull_request_reviews {
            info!(
                "Processing review {} for PR {}({})",
                pull_request_review.id(),
                pull_request.id(),
                pull_request.title()
            );

            if pull_request_review.state() == "APPROVED" {
                has_approved_reviews = true;
            }
        }

        if !has_approved_reviews && !has_reviews_with_outstanding_comments {
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
    let repositories: Vec<&str> = repositories.split(",").collect();
    let github_token: String = env::var("GITHUB_TOKEN").context("GITHUB_TOKEN must be set")?;
    let webhook_url: String =
        env::var("GOOGLE_WEBHOOK_URL").context("GOOGLE_WEBHOOK_URL must be set")?;
    let ignored_users: String = env::var("GITHUB_IGNORED_USERS").unwrap_or("".to_string());
    let ignored_users: Vec<&str> = ignored_users.split(",").collect();
    let ignored_labels: String = env::var("GITHUB_IGNORED_LABELS").unwrap_or("".to_string());
    let ignored_labels: Vec<&str> = ignored_labels.split(",").collect();

    let mut pull_requests_to_review: Vec<GithubPullRequest> = vec![];

    for repository in repositories {
        pull_requests_to_review.append(
            &mut scan_repository(
                repository.to_string(),
                &github_token,
                &ignored_users,
                &ignored_labels,
            )
            .await?,
        );
    }

    if pull_requests_to_review.len() > 0 {

        let weekday = match chrono::offset::Local::now().date().weekday() {
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
        message.push_str("--------------------\n\n");
        message.push_str("This message is brought to you by https://github.com/guardian/actions-prnouncer, ");
        message.push_str("with configuration from https://github.com/guardian/prnouncer-config\n");
        message.push_str("--------------------\n\n");

        let thread_key = format!("pr-thread-{}", chrono::offset::Local::now());

        info!("Using thread key {}", thread_key);

        GoogleChatMessage::from(message)
            .send(&webhook_url, &thread_key)
            .await?;

        for pull_request in pull_requests_to_review {
            GoogleChatMessage::from(format!(
                "<{}|{}#{}> - {}\n",
                pull_request.html_url(),
                pull_request.head().repo().name(),
                pull_request.number(),
                pull_request.title()
            ))
            .send(&webhook_url, &thread_key)
            .await?;
        }
    }
    else {
    info!("No open PRs found, no action taken.");
    }

    Ok(())
}

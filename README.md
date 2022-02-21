# Google Chats PR Announcer

This action automatically sends a message to google chats detailing the list of un-reviewed PR's. It can be configured to run daily on a cron job.

## Inputs

## `google-webhook-url`

**Required** The webhook URL (including the secret key) to send messages to, this can be gotten from google chats.

## `github-repositories`

**Required** Comma delimited list of repository owners (eg guardian/dotcom-rendering). Defaults to `github.repository`

## `github-token`

**Required** Github token used to authenticate with Github API. Defaults to `github.token`

## `github-ignored-users`

**Required** Github users to ignore when scanning for PR's. Defaults to `49699333` (Dependabot)

## `github-ignored-labels`

**Required** PR labels to ignore when scanning for PR's. Defaults to `Stale`

## Example usage

uses: guardian/google-chats-pr-announcer@v1
with:
google-webhook-url: 'https://chats.google.com...'

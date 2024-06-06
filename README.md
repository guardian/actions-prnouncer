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

## `github-announced-users`

**Optional** Only Github users to announce PR's from. If set, other users' PR's will be ignored.

## Example usage

```yaml
uses: guardian/actions-prnouncer@v1
  with:
    google-webhook-url: 'https://chats.google.com...'
```

## Contributing

### Running Locally

```bash
# List of repositories to scan
export GITHUB_REPOSITORIES=guardian/dotcom-rendering,guardian/frontend
# Token used for accessing github API's, this should be your personal access token
export GITHUB_TOKEN=<secret!>
# Webhook URL to send chat messages to, can be generated in the Google Chats application.
export GOOGLE_WEBHOOK_URL=https://chats.google.com...
# List of users to ignore when scanning for PR's, specified by user id.
# (e.g. 49699333 is dependabot)
export GITHUB_IGNORED_USERS=49699333
# List of labels to ignore when scanning for PR's
export GITHUB_IGNORED_LABELS=dependencies
# List of users to announce PR's from (if set, other users will be ignored)
# (e.g. 49699333 is dependabot, 19733683 is snykbot, 108136057 is scala steward)
export GITHUB_ANNOUNCED_USERS=49699333,19733683,108136057

cargo run
```

### Running Locally in VSCode

A VSCode run task has been included for simplicity. You can run the project by following these steps

1.  Press `Ctrl` + `Shift` + `P`
2.  Search for `Preferences: Open Settings (JSON)`
3.  Add your secret values to your vscode settings (replace linux with osx/windows as needed)

```json
{
  "terminal.integrated.env.linux": {
    "GITHUB_TOKEN": "Super secret!",
    "GOOGLE_WEBHOOK_URL": "Also secret!"
  }
}
```

4.  Search for `Tasks: Run Task`
5.  Select `Run google-chats-pr-announcer`

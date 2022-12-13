use anyhow::{bail, Context as _, Result};
use base64::{decode, encode};
use regex::Regex;
use reqwest::{header, header::HeaderValue, Client, StatusCode};
use serde::Deserialize;
use serde_json::json;
use serenity::{
    client::Context, futures::StreamExt,
    model::application::interaction::application_command::ApplicationCommandInteraction,
    model::application::interaction::InteractionResponseType,
};
use std::collections::HashMap;

const SIGNATURE: &str = "<!-- DISCORD_BOT_FAQ - DO NOT REMOVE -->";

#[derive(Default)]
struct GitHubAPI {
    origin_api_root: String,
    upstream_api_root: String,
    client: Client,
    token: String,
    origin_work_branch_name: String,
    upstream_main_branch_name: String,
    upstream_user_name: String,
    origin_user_name: String,
}

#[derive(Deserialize)]
struct RepoBranch {
    object: RepoBranchObject,
}

#[derive(Deserialize)]
struct RepoBranchObject {
    sha: String,
}

#[derive(Deserialize)]
struct RepoFile {
    sha: String,
    path: String,
    content: String,
}

#[derive(Deserialize)]
struct GitHubPullReqObj {
    html_url: String,
}

impl GitHubAPI {
    fn from(self) -> Self {
        let client = Client::builder()
            .default_headers(
                [
                    (header::USER_AGENT, HeaderValue::from_static("Optimus")),
                    (
                        header::AUTHORIZATION,
                        format!("Bearer {}", self.token)
                            .parse()
                            .expect("Can't parse token"),
                    ),
                    (
                        header::ACCEPT,
                        HeaderValue::from_static("application/vnd.github+json"),
                    ),
                ]
                .into_iter()
                .collect(),
            )
            .build()
            .expect("Can't build http client");

        Self {
            origin_api_root: self.origin_api_root,
            upstream_api_root: self.upstream_api_root,
            token: self.token,
            client,
            origin_work_branch_name: self.origin_work_branch_name,
            upstream_main_branch_name: self.upstream_main_branch_name,
            upstream_user_name: self.upstream_user_name,
            origin_user_name: self.origin_user_name,
        }
    }

    async fn sync_fork_from_upstream(
        &self,
        // owner: &str,
        // repo: &str,
        branch: &str,
    ) -> Result<reqwest::Response, reqwest::Error> {
        self.client
            .post(format!("{}/merge-upstream", &self.origin_api_root))
            .json(&HashMap::from([("branch", branch)]))
            .send()
            .await
    }

    async fn create_or_delete_branch(
        &self,
        main_branch: &str,
        /* owner: &str, repo: &str, */ branch: &str,
        action: &str,
    ) -> Result<()> {
        let get_branch = self
            .client
            .get(format!("{}/branches/{branch}", &self.origin_api_root))
            .send()
            .await?;

        // Only create if the branch doesn't exist
        match action {
            "create" => {
                if get_branch.status().eq(&StatusCode::NOT_FOUND) {
                    let main_branch_sha = self
                        .client
                        .get(format!(
                            "{}/git/refs/heads/{main_branch}",
                            &self.origin_api_root
                        ))
                        .send()
                        .await?
                        .json::<RepoBranch>()
                        .await?;

                    let _ = self
                        .client
                        .post(format!("{}/git/refs", &self.origin_api_root))
                        .json(&HashMap::from([
                            ("ref", "refs/heads/".to_owned() + branch),
                            ("sha", main_branch_sha.object.sha),
                        ]))
                        .send()
                        .await?;

                    // if !create_branch.status().eq(&StatusCode::OK) {
                    //     bail!("Can't create branch");
                    // }
                }
            }
            "delete" => {
                if get_branch.status().eq(&StatusCode::OK) {
                    let _ = self
                        .client
                        .delete(format!("{}/git/refs/heads/{branch}", &self.origin_api_root))
                        .send()
                        .await?;
                }
            }

            _ => {}
        }

        Ok(())
    }

    async fn get_file(&self, path: &str, branch: &str) -> Result<RepoFile> {
        let req = self
            .client
            .get(format!("{}/contents/{path}", &self.origin_api_root))
            .query(&[("ref", branch)])
            .send()
            .await?;

        let req = req.json::<RepoFile>().await?;
        Ok(req)
    }

    async fn commit(
        &self,
        path: &str,
        message: &str,
        committer_name: &str,
        committer_email: &str,
        content: &str,
        original_sha: &str,
        branch: &str,
    ) -> Result<reqwest::Response> {
        let req = self
            .client
            .put(format!("{}/contents/{path}", &self.origin_api_root))
            .json(&json!({
                "message": message,
                "committer": {
                    "name": committer_name,
                    "email": committer_email,
                },
                "content": content,
                "sha": original_sha,
                "branch": branch,
            }))
            .send()
            .await?;

        Ok(req)
    }

    async fn get_origin_pr_on_upstream(&self) -> Result<String> {
        if let Ok(value) = self
            .client
            .get(format!("{}/pulls", &self.upstream_api_root))
            .query(&[
                ("state", "open"),
                (
                    "head",
                    format!(
                        "{}:{}",
                        &self.origin_user_name, &self.origin_work_branch_name
                    )
                    .as_str(),
                ),
            ])
            .send()
            .await?
            .json::<Vec<GitHubPullReqObj>>()
            .await
        {
            let first = value.first();
            if first.is_some() {
                return Ok(String::from(&first.context("Cant get first")?.html_url));
            }
        }

        bail!("Couldn't fetch open PRs on upstream");
    }

    async fn pull_request(
        &self,
        title: &str,
        body: &str,
        head: &str,
        base: &str,
    ) -> Result<String> {
        let req = self
            .client
            .post(format!("{}/pulls", &self.upstream_api_root))
            .json(&json!({
                "title": title,
                "body": body,
                "base": base,
                "head": head,
                "maintainer_can_modify": true,
                "draft": true,
            }))
            .send()
            .await?;

        Ok(req.json::<GitHubPullReqObj>().await?.html_url)
    }
}

pub async fn responder(mci: &ApplicationCommandInteraction, ctx: &Context) -> Result<()> {
    let options = &mci.data.options;

    let title = &options
        .get(0)
        .context("Failed to get title")?
        .value
        .as_ref()
        .context("Error getting value")?
        .to_string();
    let title = title.trim_start_matches('"').trim_end_matches('"');

    let link = &options
        .get(1)
        .context("Failed to get link")?
        .value
        .as_ref()
        .context("Error getting value")?
        .to_string();
    let link = link.trim_start_matches('"').trim_end_matches('"');

    mci.create_interaction_response(&ctx.http, |r| {
        r.kind(InteractionResponseType::DeferredChannelMessageWithSource)
    })
    .await?;

    let mut sanitized_messages: Vec<String> = Vec::new();
    let mut messages_iter = mci.channel_id.messages_iter(&ctx.http).boxed();

    while let Some(message_result) = messages_iter.next().await {
        if let Ok(message) = message_result {
            // Skip if bot
            if message.author.bot {
                continue;
            }

            let attachments = &message
                .attachments
                .into_iter()
                .map(|a| format!("{}\n", a.url))
                .collect::<String>();

            let content = Regex::new(r#"<(?:a:\w+:)?(?:@|(?:@!)|(?:@&)|#)\d+>"#)?
                .replace_all(message.content.as_str(), "<redacted>")
                .to_string();
            let content = Regex::new(r#"```"#)?.replace(content.as_str(), "\n```");

            sanitized_messages.push(format!(
                "\n**{}{}**: {}\n{attachments}",
                message.author.name, message.author.discriminator, content
            ));
        }
    }

    sanitized_messages.push(format!("### {}\n{}\n", title, SIGNATURE));
    sanitized_messages.reverse();
    let sanitized_messages = sanitized_messages.into_iter().collect::<String>();

    let github_client = GitHubAPI::from(GitHubAPI {
        origin_api_root: "https://api.github.com/repos/github-activity/website".to_owned(),
        upstream_api_root: "https://api.github.com/repos/gitpod-io/website".to_owned(),
        token: obfstr::obfstr!(env!("GITHUB_TOKEN")).to_owned(),
        upstream_main_branch_name: "main".to_owned(),
        upstream_user_name: "gitpod-io".to_owned(),
        origin_work_branch_name: "discord_staging".to_owned(),
        origin_user_name: "github-activity".to_owned(),
        ..Default::default()
    });

    let relative_file_path = Regex::new(r#"^.*/docs/"#)?.replace(link, "gitpod/docs/");

    // Sync fork
    github_client
        .sync_fork_from_upstream(github_client.upstream_main_branch_name.as_str())
        .await?;

    if github_client.get_origin_pr_on_upstream().await.is_err() {
        // Delete branch if no PR is open in upstream
        github_client
            .create_or_delete_branch(
                github_client.upstream_main_branch_name.as_str(),
                github_client.origin_work_branch_name.as_str(),
                "delete",
            )
            .await
            // Ignore any error.
            .ok();

        // Create branch
        github_client
            .create_or_delete_branch(
                github_client.upstream_main_branch_name.as_str(),
                github_client.origin_work_branch_name.as_str(),
                "create",
            )
            .await?;
    }

    // Committing the changes
    /////////////////////////

    // Get file object
    let file = {
        if let Ok(result) = github_client
            .get_file(
                format!("{relative_file_path}.md").as_str(),
                github_client.origin_work_branch_name.as_str(),
            )
            .await
        {
            result
        } else {
            github_client
                .get_file(
                    format!("{relative_file_path}/index.md").as_str(),
                    github_client.origin_work_branch_name.as_str(),
                )
                .await?
        }
    };

    // Prepare new file contents
    let file_contents_decoded = {
        let decoded = decode(file.content.split_whitespace().collect::<String>())?;
        let decoded = String::from_utf8(decoded)?;

        // Append to FAQs
        if decoded.contains("FAQs") {
            Regex::new("FAQs")?
                .replace(decoded.as_str(), format!("FAQs\n\n{sanitized_messages}"))
                .to_string()
        } else {
            format!("{decoded}\n\n## FAQs\n\n{sanitized_messages}")
        }
    };

    // Base64 encode
    let file_contents_encoded = encode(file_contents_decoded);

    // Commit the new changes
    github_client
        .commit(
            file.path.as_str(),
            format!("Update {}", file.path).as_str(),
            "Robo Quat",
            "roboquat@gitpod.io",
            file_contents_encoded.as_str(),
            file.sha.as_str(),
            github_client.origin_work_branch_name.as_str(),
        )
        .await?;

    // Create PR
    let pr_link = {
        let pr = github_client.get_origin_pr_on_upstream().await;

        if pr.is_ok() {
            pr?
        } else {
            github_client
                .pull_request(
                    format!("Add FAQ for {relative_file_path}").as_str(),
                    "Pulling a Discord thread as FAQ",
                    format!(
                        "{}:{}",
                        github_client.origin_user_name, github_client.origin_work_branch_name,
                    )
                    .as_str(),
                    github_client.upstream_main_branch_name.as_str(),
                )
                .await?
        }
    };

    mci.edit_original_interaction_response(&ctx.http, |r| {
        r.content(format!("PR for this thread conversation: {pr_link}"))
    })
    .await?;

    Ok(())
}

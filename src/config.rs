use color_eyre::{eyre::Context, Report};
use serde::Deserialize;
use serenity::model::prelude::ChannelId;

// Top level
#[derive(Debug, Deserialize)]
pub struct BotConfig {
    pub github: Option<GithubConfig>,
    pub discord: DiscordConfig,
    pub meilisearch: Option<MeilisearchConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GithubConfig {
    pub api_token: String,
    pub user_agent: String,
}

#[derive(Debug, Deserialize)]
pub struct DiscordConfig {
    pub application_id: u64,
    pub bot_token: String,
    pub channels: Option<DiscordChannels>,
}

#[derive(Debug, Deserialize)]
pub struct DiscordChannels {
    pub introduction_channel_id: Option<ChannelId>,
    pub general_channel_id: Option<ChannelId>,
    pub getting_started_channel_id: Option<ChannelId>,
    pub off_topic_channel_id: Option<ChannelId>,
    pub primary_questions_channel_id: Option<ChannelId>,
    pub secondary_questions_channel_id: Option<ChannelId>,
}

#[derive(Debug, Deserialize)]
pub struct MeilisearchConfig {
    pub api_key: String,
    pub api_endpoint: String,
}

pub fn read(toml_path: &str) -> Result<BotConfig, Report> {
    // Read the TOML file into a var
    let contents = std::fs::read_to_string(toml_path)
        .wrap_err_with(|| format!("Couldn't read a {toml_path} file from the provided path"))?;

    // Parse the TOML string into a `Config` object
    let config: BotConfig =
        toml::from_str(&contents).wrap_err_with(|| format!("Failed to parse {toml_path}"))?;

    // Return
    Ok(config)
}

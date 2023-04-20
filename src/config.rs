use std::path::Path;

use color_eyre::{
    eyre::{eyre, Context},
    Report,
};
use serde::Deserialize;
use serenity::model::prelude::ChannelId;

// Top level
#[derive(Debug, Deserialize)]
pub struct BotConfig {
    pub github: Option<GithubConfig>,
    pub discord: DiscordConfig,
    pub meilisearch: Option<MeilisearchConfig>,
    pub openai: Option<OpenaiConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GithubConfig {
    pub api_token: String,
    pub user_agent: String,
}

#[derive(Debug, Deserialize)]
pub struct DiscordConfig {
    // pub application_id: u64,
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
    pub master_key: String,
    pub api_endpoint: String,
    pub server_cmd: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct OpenaiConfig {
    pub api_key: String,
}

pub fn read(toml_path: &Path) -> Result<BotConfig, Report> {
    // Get executable path and parent dir
    let exec_path = std::env::current_exe()?;
    let exec_parent_dir = exec_path
        .parent()
        .ok_or_else(|| eyre!("Failed to get parent dir of self"))?;
    let exec_parent_dir_config = exec_parent_dir.join("BotConfig.toml");

    // Read the TOML file into a var
    let contents = [toml_path, &exec_parent_dir_config]
        .iter()
        .find_map(|path| std::fs::read_to_string(path).ok())
        .ok_or_else(|| eyre!("Failed to read a BotConfig"))?;

    // Parse the TOML string into a `Config` object
    let config: BotConfig =
        toml::from_str(&contents).wrap_err_with(|| format!("Failed to parse {:?}", toml_path))?;

    // Return
    Ok(config)
}

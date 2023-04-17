use crate::init::MEILICLIENT_THREAD_INDEX;
use color_eyre::eyre::{eyre, Report, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};

use serenity::{
    client::Context,
    futures::StreamExt,
    model::{
        id::ChannelId,
        prelude::{GuildChannel, MessageType},
        Timestamp,
    },
    utils::{content_safe, ContentSafeOptions},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Thread {
    pub title: String,
    pub messages: Vec<String>,
    pub tags: Vec<String>,
    pub author_id: u64,
    pub id: u64,
    pub guild_id: u64,
    pub parent_channel_id: u64,
    pub timestamp: i64,
    pub date: Timestamp,
    pub poster: String, // author discord avatar
}

pub async fn index_channel_threads(ctx: &Context, channel_ids: &[ChannelId]) -> Result<(), Report> {
    // let channel_id = ChannelId(1026115789721444384);
    // let guild_id = GuildId(947769443189129236);

    for channel_id in channel_ids {
        // Get archived threads from channel_id
        let archived_threads = channel_id
            .get_archived_public_threads(&ctx.http, None, None)
            .await?
            .threads;

        // Iterate over archived threads (AKA posts) from the (forum) channel
        index_thread_messages(ctx, &archived_threads).await?;
    }

    Ok(())
}

pub async fn index_thread_messages(
    ctx: &Context,
    threads: &Vec<GuildChannel>,
) -> Result<(), Report> {
    for thread in threads {
        // Gather some data about the thread
        let thread_node = thread
            .id
            .to_channel(&ctx.http)
            .await?
            .guild()
            .ok_or_else(|| eyre!("Failed to get thread node"))?;
        let thread_id = thread_node.id;
        let guild_id = thread_node.guild_id;
        let thread_parent_channel_id = thread_node
            .parent_id
            .ok_or_else(|| eyre!("Failed to get parent_id of thread"))?;
        let thread_title = thread_node.name;
        let thread_author_id = thread_node
            .owner_id
            .ok_or_else(|| eyre!("Failed to get owner_id of thread"))?;
        let thread_author_avatar_url = guild_id
            .member(&ctx.http, thread_author_id)
            .await?
            .user
            .face();
        // Get tags
        // TODO: How to optimize this, and better visualize this problem in mind, ask Thomas.
        let thread_available_tags = thread_parent_channel_id
            .to_channel(&ctx.http)
            .await?
            .guild()
            .ok_or_else(|| eyre!("Fauled to get parent guild channel"))?
            .available_tags;

        let thread_tags = thread_node
            .applied_tags
            .into_iter()
            .filter_map(|tag| thread_available_tags.iter().find(|avt| avt.id == tag))
            .map(|x| x.name.to_owned())
            .collect::<Vec<String>>();

        let thread_timestamp = {
            let meta = thread_node
                .thread_metadata
                .ok_or_else(|| eyre!("Cant fetch metadata"))?;
            if let Some(time) = meta.create_timestamp {
                time
            } else if let Some(time) = meta.archive_timestamp {
                time
            } else {
                thread_node.id.created_at()
            }
        };

        // Get thread users
        /* let thread_user_ids: Vec<UserId> = thread_id
            .get_thread_members(&ctx.http)
            .await?
            .iter()
            .filter_map(|m| m.user_id)
            .collect();

        let mut thread_users: Vec<User> = Vec::new();
        for thread_member in thread_user_ids {
            if let Ok(member) = guild_id.member(&ctx.http, thread_member).await {
                thread_users.push(member.user);
            }
        } */

        // loop inside thread
        let mut sanitized_messages: Vec<String> = Vec::new();
        let mut thread_messages_iter = thread_id.messages_iter(&ctx.http).boxed();
        while let Some(message_item) = thread_messages_iter.next().await && let Ok(message) = message_item {

            // Skip if bot or system
            if message.author.bot || message.kind.eq(&MessageType::GroupNameUpdate) {
                continue;
            }

            // Collect the attachments
            let attachments = &message
                .attachments
                .into_iter()
                .map(|a| format!("{}\n", a.url))
                .collect::<String>();

            let content = content_safe(&ctx.cache, &message.content, &ContentSafeOptions::default(), &[]);
            let content = Regex::new(r#"```"#)?.replace(&content, "\n```");

            if attachments.is_empty() {
                sanitized_messages.push(format!(
                    "**{}#{}**: {content}",
                    message.author.name, message.author.discriminator
                ));
            } else {
                sanitized_messages.push(format!(
                    "**{}#{}**: {content}\n{attachments}",
                    message.author.name, message.author.discriminator
                ));
            }
        }

        // Fix message order
        sanitized_messages.reverse();

        MEILICLIENT_THREAD_INDEX
            .get()
            .ok_or_else(|| eyre!("Failed to get meiliclient"))?
            .add_documents(
                &[Thread {
                    title: thread_title,
                    messages: sanitized_messages,
                    tags: thread_tags,
                    author_id: thread_author_id.into(),
                    id: thread_id.into(),
                    guild_id: *guild_id.as_u64(),
                    parent_channel_id: thread_parent_channel_id.into(),
                    timestamp: thread_timestamp.unix_timestamp(),
                    date: thread_timestamp,
                    poster: thread_author_avatar_url,
                }],
                Some("id"),
            )
            .await?;
    }
    Ok(())
}

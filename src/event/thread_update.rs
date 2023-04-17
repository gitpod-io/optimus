use crate::utils::index_threads::index_thread_messages;
use color_eyre::eyre::{eyre, Result};
use regex::Regex;
use serenity::{client::Context, model::channel::GuildChannel, model::channel::MessageType};

// Was trying to hook into auto thread archival and ask the participants
// if the thread was resolved but guess we can't reliably do it now
// since there is no reliable way to detect who triggered thread_update
// Tl;dr : Discord API doesn't tell you who archived the thread, which is a big issue.
async fn unarchival_action(_ctx: Context, _thread: GuildChannel) -> Result<()> {
    // _thread
    //         .say(
    //             &_ctx.http,
    //             "Whoever is trying to archive from the Discord UI, please send `/close` as a message here instead.",
    //         )
    //         .await
    //         ?;
    _thread
        .edit_thread(&_ctx.http, |t| t.archived(false))
        .await?;

    Ok(())
}

pub async fn responder(ctx: Context, thread: GuildChannel) -> Result<()> {
    if let Some(config) = crate::BOT_CONFIG.get() && let Some(channels) = &config.discord.channels
    && let Some(primary_questions_channel) = channels.primary_questions_channel_id
    && let Some(secondary_questions_channel) = channels.secondary_questions_channel_id  {

        let thread_type = {
            if [primary_questions_channel, secondary_questions_channel].contains(
                &thread
                    .parent_id
                    .ok_or_else(|| eyre!("Couldn't get parent_id of thread"))?,
            ) {
                "question"
            } else {
                "thread"
            }
        };
        let last_msg = &ctx.http.get_messages(*thread.id.as_u64(), "").await?;
        let last_msg = last_msg
            .first()
            .ok_or_else(|| eyre!("Couldn't get last message"))?;

        if thread_type == "question" {
            // Index to DB
            index_thread_messages(&ctx, &vec![thread.clone()])
                .await
                .ok();

            if thread
                .thread_metadata
                .ok_or_else(|| eyre!("Couldn't get thread_metadata"))?
                .archived
                && last_msg.is_own(&ctx.cache)
            {
                if !(last_msg.kind.eq(&MessageType::GroupNameUpdate)
                    || Regex::new("^This [a-z]+ was closed ?b?y?")?.is_match(last_msg.content.as_str()))
                {
                    unarchival_action(ctx, thread).await?;
                }
            } else if thread
                .thread_metadata
                .ok_or_else(|| eyre!("Couldn't get thread_metadata"))?
                .archived
            {
                unarchival_action(ctx, thread).await?;
            }
        }
    }

    Ok(())
}

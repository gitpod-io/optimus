use regex::Regex;
use serenity::model::channel::MessageType;

use super::*;

// Was trying to hook into auto thread archival and ask the participants
// if the thread was resolved but guess we can't reliably do it now
// since there is no reliable way to detect who triggered thread_update
// Tl;dr : Discord API doesn't tell you who archived the thread, which is a big issue.
async fn unarchival_action(_ctx: Context, _thread: GuildChannel) {
    _thread
            .say(
                &_ctx.http,
                "Whoever is trying to archive from the Discord UI, please send `/close` as a message here instead.",
            )
            .await
            .unwrap();
    _thread
        .edit_thread(&_ctx.http, |t| t.archived(false))
        .await
        .unwrap();
}

pub async fn responder(_ctx: Context, _thread: GuildChannel) {
    // let thread_type = {
    //     if _thread.name.starts_with("✅") || _thread.name.starts_with("❓") {
    //         "question"
    //     } else {
    //         "thread"
    //     }
    // };
    let last_msg = &_ctx
        .http
        .get_messages(*_thread.id.as_u64(), "")
        .await
        .unwrap();
    let last_msg = last_msg.first().unwrap();

    if _thread.thread_metadata.unwrap().archived && last_msg.is_own(&_ctx.cache).await {
        if last_msg.kind.eq(&MessageType::GroupNameUpdate)
            || Regex::new(format!("^This [a-z]+ was closed ?b?y?").as_str())
                .unwrap()
                .is_match(last_msg.content.as_str())
        {
            return;
        } else {
            unarchival_action(_ctx, _thread).await;
        }
    } else if _thread.thread_metadata.unwrap().archived {
        unarchival_action(_ctx, _thread).await;
    }
}

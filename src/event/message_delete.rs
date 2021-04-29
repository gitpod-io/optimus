use super::*;

pub async fn responder(
    _ctx: Context,
    _channel_id: ChannelId,
    _deleted_message_id: MessageId,
    _guild_id: Option<GuildId>,
) {
    let dbnode = Database::from("msgcache".to_string()).await;
    let deleted_message = dbnode.fetch_deleted_msg(_deleted_message_id).await;

    if !Regex::new(r"^.react")
        .unwrap()
        .is_match(&deleted_message.as_str())
        && !Regex::new(r"^dsay ")
            .unwrap()
            .is_match(&deleted_message.as_str())
        && !Regex::new(r":*:")
            .unwrap()
            .is_match(&deleted_message.as_str())
        && !Regex::new(r"^.delete")
            .unwrap()
            .is_match(&deleted_message.as_str())
    {
        let settings = {
            ContentSafeOptions::default()
                .clean_channel(false)
                .clean_role(true)
                .clean_user(false)
                .clean_everyone(true)
                .clean_here(true)
        };

        let content = content_safe(
            &_ctx.cache,
            &deleted_message.replace("~~MSG_TYPE~~", "Deleted:"),
            &settings,
        )
        .await;

        _channel_id.say(&_ctx, &content).await.ok();
        process::Command::new("find")
            .args(&[
                dbnode.to_string(),
                String::from("-type"),
                String::from("f"),
                String::from("-mtime"),
                String::from("+5"),
                String::from("-delete"),
            ])
            .spawn()
            .ok();
    }
}

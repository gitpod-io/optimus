use super::*;

pub async fn responder(
    _ctx: Context,
    _channel_id: ChannelId,
    _deleted_message_id: MessageId,
    _guild_id: Option<GuildId>,
) {
    let dbnode = Database::from("msgcache".to_string()).await;
    let deleted_message = dbnode.fetch_deleted_msg(_deleted_message_id).await;

    // let last_msg_id = _new
    //     .unwrap()
    //     .channel(&_ctx.cache)
    //     .await
    //     .unwrap()
    //     .guild()
    //     .unwrap()
    //     .last_message_id
    //     .unwrap();

    // let last_msg_id = _ctx
    //     .cache
    //     .channel(_channel_id)
    //     .await
    //     .unwrap()
    //     .guild()
    //     .unwrap()
    //     .last_message_id
    //     .unwrap();

    let qq = _ctx
        .http
        .get_messages(u64::try_from(_channel_id).unwrap(), "")
        .await
        .unwrap();

    let gg = _ctx.cache.guild(_guild_id.unwrap()).await.unwrap();

    let nqn_exists = gg.member(&_ctx.http, 559426966151757824).await;

    let botis = &qq.first().as_ref().map(|x| x.author.id).unwrap();

    let is_valid_member = gg.member(&_ctx.http, botis).await;

    let re = Regex::new("<*.:.*.:.*>").unwrap();
    let re2 = Regex::new(":.*:").unwrap();
    let re3 = Regex::new("\\n.* ~~MSG_TYPE~~.*").unwrap();

    let mut parsed_last_msg = re
        .replace_all(
            &qq.first()
                .as_ref()
                .map(|x| String::from(dbg!(&x.content)))
                .unwrap(),
            "%%emoji%%",
        )
        .to_string();

    parsed_last_msg = re2.replace_all(&parsed_last_msg, "%%emoji%%").to_string();

    let mut parsed_deleted_msg = re
        .replace_all(dbg!(&deleted_message).as_str(), "%%emoji%%")
        .to_string();
    parsed_deleted_msg = re2
        .replace_all(&parsed_deleted_msg, "%%emoji%%")
        .to_string();

    parsed_deleted_msg = re3.replace_all(&parsed_deleted_msg, "").to_string();

    let msg_is_nqnbot = {
        if nqn_exists.is_err() {
            false
        } else if is_valid_member.is_err() {
            if Regex::new(format!("^{}$", dbg!(parsed_last_msg)).as_str())
                .unwrap()
                .is_match(dbg!(&parsed_deleted_msg).as_str())
            // if dbg!(parsed_last_msg).contains(dbg!(&parsed_deleted_msg))
            {
                dbg!("hmm");
                true
            } else {
                false
            }
        } else {
            false
        }
    };

    // let botis = _ctx
    //     .cache
    //     .message(_channel_id, last_msg_id)
    //     .await
    //     .unwrap()
    //     .author
    //     .bot;

    if dbg!(!msg_is_nqnbot)
        && !Regex::new(r"^.react")
            .unwrap()
            .is_match(&deleted_message.as_str())
        && !Regex::new(r"^dsay ")
            .unwrap()
            .is_match(&deleted_message.as_str())
        // && !Regex::new(r":*:")
        //     .unwrap()
        //     .is_match(&deleted_message.as_str())
        && !Regex::new(r"^.delete")
            .unwrap()
            .is_match(&deleted_message.as_str())
    {
        let settings = {
            ContentSafeOptions::default()
                .clean_channel(false)
                .clean_role(true)
                .clean_user(true)
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

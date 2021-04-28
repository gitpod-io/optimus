use super::*;

pub async fn responder(_ctx: Context, mut _msg: Message) {
    // let map = json!({"name": "test"});
    // let channel_id = _msg.channel_id.borrow().clone();

    // let mut _webhook = &_ctx
    //     .http
    //     .create_webhook(u64::try_from(channel_id).unwrap(), &map)
    //     .await;

    // let webhook_id = u64::try_from(&_webhook.map(|x| x.id).unwrap()).unwrap();
    // // let webhook_token = format!("{:?}", _webhook.map(|x| x.token).unwrap()).as_str();
    // let mut webhook = &_ctx.http.get_webhook_with_token(webhook_id, "").await;
    // // _webhook.map(|x| x.token);

    // let user_date = _new_member.user.created_at().date().naive_utc();
    // let dbnode_userid = Database::from("userid".to_string()).await;
    // let members = &_msg.guild(&_ctx.cache).await.unwrap().members;

    // for (_user_id, _member) in members {
    //     dbnode_userid
    //         .save_user_info(&_user_id, _member.user.tag())
    //         .await;
    // }

    if !_msg.is_own(&_ctx.cache).await {
        let dbnode_msgcache = Database::from("msgcache".to_string()).await;

        if !Regex::new(r"^.react ")
            .unwrap()
            .is_match(&_msg.content.as_str())
            && !Regex::new(r"^dsay ")
                .unwrap()
                .is_match(&_msg.content.as_str())
            && !Regex::new(r":*:").unwrap().is_match(&_msg.content.as_str())
            && !Regex::new(r"^.delete ")
                .unwrap()
                .is_match(&_msg.content.as_str())
        {
            let attc = &_msg.attachments;
            let mut _attachments = String::new();

            for var in attc.iter() {
                let url = &var.proxy_url;
                _attachments.push_str(format!("\n{}", url).as_str());
            }

            // let v: Value = serde_json::from_str(&_msg.attachments.iter().map(|x| x.proxy_url.as_str())).unwrap();
            dbnode_msgcache
                .cache_deleted_msg(
                    &_msg.id,
                    format!(
                        "{}{}\n> ~~MSG_TYPE~~ {}",
                        &_msg.content,
                        &_attachments,
                        &_msg.author,
                        // &_msg.timestamp
                    ),
                )
                .await;
        }
    }

    let dbnode_notes = Database::from("notes".to_string()).await;
    let ref_msg = &_msg.referenced_message;

    let options = MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };
    if !_msg.author.bot && !_msg.content.contains("dnote ") {
        for entry in glob_with(format!("{}/*", dbnode_notes).as_str(), options).unwrap() {
            match entry {
                Ok(path) => {
                    let note = path.file_name().unwrap().to_string_lossy().to_string();

                    if _msg
                        .content
                        .to_lowercase()
                        .contains(&note.as_str().to_lowercase())
                    {
                        let typing = _ctx
                            .http
                            .start_typing(u64::try_from(_msg.channel_id).unwrap())
                            .unwrap();
                        let content = Note::from(&note).await.get_contents().await;
                        if ref_msg.is_some() {
                            &ref_msg
                                .as_ref()
                                .map(|x| x.reply_ping(&_ctx.http, &content))
                                .unwrap()
                                .await
                                .unwrap()
                                .react(&_ctx.http, '❎')
                                .await
                                .unwrap();
                        } else {
                            _msg.reply(&_ctx.http, &content)
                                .await
                                .unwrap()
                                .react(&_ctx.http, '❎')
                                .await
                                .unwrap();
                        }
                        typing.stop();
                    }
                }
                Err(e) => println!("{:?}", e),
            }
        }
    }

    // let user_date = &_msg.author.created_at().naive_utc().date();
    // let user_time = &_msg.author.created_at().naive_utc().time();
    // let _how_old = "";
    // let _system_channel_id = _ctx
    //     .cache
    //     .guild(&_msg.guild_id.unwrap())
    //     .await
    //     .map(|x| x.system_channel_id)
    //     .unwrap()
    //     .unwrap();

    // _ctx.http
    //     .send_message(
    //         u64::try_from(_system_channel_id).unwrap(),
    //         &json!({
    //             "content":
    //                 format!(
    //                     "> :arrow_forward: {}'s account Date: **{}**; Time: **{}**",
    //                     &_msg.author, &user_date, &user_time
    //                 )
    //         }),
    //     )
    //     .await
    //     .unwrap();
}

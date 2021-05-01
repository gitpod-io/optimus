use super::*;

pub async fn responder(
    _ctx: Context,
    _channel_id: ChannelId,
    _deleted_message_id: MessageId,
    _guild_id: Option<GuildId>,
) {
    let dbnode = Database::from("msgcache".to_string()).await;
    let deleted_message = dbnode.fetch_msg(_deleted_message_id).await;

    // let last_msg_id = _new
    //     .unwrap()
    //     .channel(&_ctx.cache)
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

    let re0 = Regex::new(r"(<:|<a:)").unwrap();
    let re = Regex::new(r"\d").unwrap();
    let re2 = Regex::new("[<::>]").unwrap();
    let re3 = Regex::new("\\n.* ~~MSG_TYPE~~.*").unwrap();

    let mut parsed_last_msg = re
        .replace_all(
            &qq.first()
                .as_ref()
                .map(|x| String::from(&x.content))
                .unwrap(),
            "",
        )
        .to_string();

    // for _ in 1..10 {
    //     parsed_last_msg = re.replace_all(&parsed_last_msg, "").to_string();
    // }

    parsed_last_msg = re0.replace_all(&parsed_last_msg, "").to_string();
    parsed_last_msg = re2.replace_all(&parsed_last_msg, "").to_string();

    let mut parsed_deleted_msg = re0.replace_all(&deleted_message.as_str(), "").to_string();
    parsed_deleted_msg = re.replace_all(&parsed_deleted_msg, "").to_string();
    parsed_deleted_msg = re2.replace_all(&parsed_deleted_msg, "").to_string();
    parsed_deleted_msg = re3.replace_all(&parsed_deleted_msg, "").to_string();

    let msg_is_nqnbot = {
        if nqn_exists.is_err() {
            false
        } else if is_valid_member.is_err() {
            if parsed_last_msg.contains(&parsed_deleted_msg)
            // if dbg!(parsed_last_msg).contains(dbg!(&parsed_deleted_msg))
            {
                // dbg!("hmm");
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

    if !msg_is_nqnbot
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

        let mut content = content_safe(
            &_ctx.cache,
            &deleted_message.replace("~~MSG_TYPE~~", "Deleted before the linked msg:"),
            &settings,
        )
        .await;

        // let mut content_urls_new = String::new();
        // let mut content_new = String::new();

        // for caps in Regex::new(r"(?P<url>https://media.discordapp.net/attachments/.*/.*\n)")
        //     .unwrap()
        //     .captures_iter(&content.as_str())
        // {
        //     let url = &caps["url"];

        //     // Check if the file is an image
        //     let mut is_image = false;
        //     let extension_var = path::Path::new(&url).extension();
        //     if extension_var.is_some() {
        //         let extension = extension_var.unwrap().to_string_lossy().to_string();

        //         match extension.as_str() {
        //             "png" | "jpeg" | "jpg" | "webp" | "gif" => {
        //                 is_image = true;
        //             }
        //             _ => {}
        //         }
        //     }

        //     if is_image {
        //         let params = [("image", url)];
        //         let client = reqwest::Client::new()
        //             .post("https://api.imgur.com/3/image")
        //             .form(&params)
        //             .header("Authorization", "Client-ID ce8c306d711c6cf")
        //             .send()
        //             .await
        //             .unwrap()
        //             .text()
        //             .await
        //             .unwrap();

        //         let client_data = Regex::new(r#"\\"#)
        //             .unwrap()
        //             .replace_all(client.as_str(), "");

        //         for caps in Regex::new(r#"(?P<link>https://.+?")"#)
        //             .unwrap()
        //             .captures_iter(&client_data)
        //         {
        //             let link = &caps["link"];

        //             content_urls_new.push_str(&Regex::new(r#"""#).unwrap().replace(&link, ""));
        //         }
        //     }
        //     content_new.push_str(&content.replace(&url, &content_urls_new));
        // }

        let last_msg = qq.first();
        let last_msg_id = last_msg.as_ref().map(|x| x.id);

        if last_msg_id.is_some() {
            let dbnode_delmsg_trigger = Database::from("delmsg_trigger".to_string()).await;
            dbnode.remove_msg(&_deleted_message_id).await;

            content = {
                if dbnode_delmsg_trigger
                    .msg_exists(&last_msg_id.unwrap())
                    .await
                {
                    let file_path = format!("{}/{}", dbnode_delmsg_trigger, &last_msg_id.unwrap());
                    let prev_content = fs::read_to_string(&file_path).await.unwrap();
                    format!("{}\n{}", &prev_content, content)
                } else {
                    content
                }
            };

            dbnode_delmsg_trigger
                .save_msg(&last_msg_id.unwrap(), content)
                .await;

            last_msg
                .as_ref()
                .map(|x| async move { x.react(&_ctx.http, '📩').await.unwrap() })
                .unwrap()
                .await;
        } else {
            _channel_id.say(&_ctx, &content).await.ok();
        }
    }

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

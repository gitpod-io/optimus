use super::*;

pub async fn responder(
    _ctx: Context,
    _channel_id: ChannelId,
    _deleted_message_id: MessageId,
    _guild_id: Option<GuildId>,
) {
    let dbnode = Database::from("msgcache".to_string()).await;

    if !dbnode.msg_exists(&_deleted_message_id).await {
        return;
    }

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

    let qq = &_ctx
        .http
        .get_messages(u64::try_from(_channel_id).unwrap(), "")
        .await
        .unwrap();

    let gg = &_ctx.cache.guild(_guild_id.unwrap()).await.unwrap();

    let nqn_exists = &gg.member(&_ctx.http, 559426966151757824).await;

    let botis = &qq.first().as_ref().map(|x| x.author.id).unwrap();

    let is_valid_member = gg.member(&_ctx.http, botis).await;

    let re0 = Regex::new(r"(<:|<a:)").unwrap();
    let re = Regex::new(r"\d").unwrap();
    let re2 = Regex::new("[<::>]").unwrap();
    let re3 = Regex::new("\\n.* ---MSG_TYPE---.*").unwrap();

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

        // Alert users who got mentioned.
        // for caps in Regex::new(r"(?P<uid>[0-9]{18}+)")
        //     .unwrap()
        //     .captures_iter(&deleted_message)
        // {
        //     let user = &caps["url"];
        //     let hmm = &_ctx.cache.member(_guild_id, user).await.unwrap();
        // }
        // End alert

        let mut content = content_safe(&_ctx.cache, &deleted_message, &settings).await;

        let mut proxied_content_attachments = Vec::new();
        let mut content_attachments = Vec::new();

        for caps in Regex::new(r"(?P<url>https://cdn.discordapp.com/attachments/.*/.*)")
            .unwrap()
            .captures_iter(&content.as_str())
        {
            let url = &caps["url"];

            content_attachments.push(caps["url"].to_string());

            // Check if the file is an image
            let mut is_image = false;
            let extension_var = path::Path::new(&url).extension();
            if extension_var.is_some() {
                let extension = extension_var.unwrap().to_string_lossy().to_string();

                match extension.as_str() {
                    "png" | "jpeg" | "jpg" | "webp" | "gif" => {
                        is_image = true;
                    }
                    _ => {}
                }
            }

            if is_image {
                let params = [("image", url)];
                let client = reqwest::Client::new()
                    .post("https://api.imgur.com/3/image")
                    .form(&params)
                    .header("Authorization", "Client-ID ce8c306d711c6cf")
                    .send()
                    .await
                    .unwrap()
                    .text()
                    .await
                    .unwrap();

                let client_data = Regex::new(r#"\\"#)
                    .unwrap()
                    .replace_all(client.as_str(), "");

                for caps_next in Regex::new(r#"(?P<link>https://.+?")"#)
                    .unwrap()
                    .captures_iter(&client_data)
                {
                    // let link = caps_next["link"];

                    proxied_content_attachments
                        .push(caps_next["link"].to_string().replace("\"", ""));

                    // println!("{}", link);

                    // content_urls_new.push_str(&Regex::new(r#"""#).unwrap().replace(&link, ""));
                }
            } else {
                let current_dir = env::current_exe().unwrap();
                let current_dir = &current_dir.parent().unwrap();
                let file_name = path::Path::new(&url)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();

                process::Command::new("curl")
                    .current_dir(&current_dir)
                    .args(&["-s", "-o", &file_name, &url])
                    .status()
                    .await
                    .unwrap();

                let client = process::Command::new("curl")
                    .current_dir(&current_dir)
                    .args(&[
                        "-F".to_string(),
                        format!("file=@{}", &file_name),
                        "-F".to_string(),
                        "no_index=false".to_string(),
                        "https://api.anonymousfiles.io".to_string(),
                    ])
                    .output()
                    .await
                    .unwrap();

                fs::remove_file(format!(
                    "{}/{}",
                    &current_dir.to_string_lossy().to_string(),
                    &file_name
                ))
                .await
                .unwrap();

                let client = String::from_utf8_lossy(&client.stdout);

                let client_data = Regex::new(r#"\\"#).unwrap().replace_all(&client, "");

                for caps_next in Regex::new(r#"(?P<link>https://.+?")"#)
                    .unwrap()
                    .captures_iter(&client_data)
                {
                    // let link = caps_next["link"];

                    proxied_content_attachments
                        .push(caps_next["link"].to_string().replace("\"", ""));

                    // content_urls_new.push_str(&Regex::new(r#"""#).unwrap().replace(&link, ""));
                }
            }
            // content_new.push_str(&content.replace(&url, &content_urls_new));
        }

        for var in proxied_content_attachments.iter() {
            content.push_str(format!("{} > `{}`", "\n", var).as_str());
            // content.replace(&content_attachments[loop_times].to_string(), &var.to_string())
        }

        let last_msg = &qq.first();
        let last_msg_id = &last_msg.as_ref().map(|x| x.id);

        if last_msg_id.is_some() {
            let dbnode_delmsg_trigger = Database::from("delmsg_trigger".to_string()).await;
            dbnode.remove_msg(&_deleted_message_id).await;

            content = {
                if dbnode_delmsg_trigger.msg_exists(&_deleted_message_id).await {
                    let file_path = format!("{}/{}", &dbnode_delmsg_trigger, &_deleted_message_id);
                    let prev_content = fs::read_to_string(&file_path).await.unwrap();
                    format!("{}\n{}", &prev_content, &content)
                } else {
                    content
                }
            };

            content = {
                if dbnode_delmsg_trigger
                    .msg_exists(&last_msg_id.unwrap())
                    .await
                {
                    let file_path = format!("{}/{}", &dbnode_delmsg_trigger, &last_msg_id.unwrap());
                    let prev_content = fs::read_to_string(&file_path).await.unwrap();
                    format!("{}\n{}", &prev_content, &content)
                } else {
                    content
                }
            };

            dbnode_delmsg_trigger
                .save_msg(&last_msg_id.unwrap(), String::from(&content))
                .await;

            last_msg
                .as_ref()
                .map(|x| async move {
                    if let Err(_) = x.react(&_ctx.http, '📩').await {
                        &_channel_id
                            .say(&_ctx, &content.replace("---MSG_TYPE---", "Deleted:"))
                            .await
                            .ok();
                    }
                })
                .unwrap()
                .await;
        } else {
            &_channel_id
                .say(&_ctx, &content.replace("---MSG_TYPE---", "Deleted:"))
                .await
                .ok();
        }
    }

    // process::Command::new("find")
    //     .args(&[
    //         dbnode.to_string(),
    //         String::from("-type"),
    //         String::from("f"),
    //         String::from("-mtime"),
    //         String::from("+5"),
    //         String::from("-delete"),
    //     ])
    //     .spawn()
    //     .ok();
}

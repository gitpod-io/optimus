use super::*;

pub async fn responder(ctx: Context, msg: Message) -> Result<()> {
    if !msg.is_own(&ctx.cache) {
        // Handle forum channel posts
        new_question::responder(&ctx, &msg).await.unwrap();

        // Log messages
        // let dbnode_msgcache = Database::from("msgcache".to_string()).await;

        // let attc = &_msg.attachments;
        // let mut _attachments = String::new();

        // for var in attc.iter() {
        //     let url = &var.url;
        //     _attachments.push_str(format!("\n{}", url).as_str());
        // }

        // let v: Value = serde_json::from_str(&_msg.attachments.iter().map(|x| x.proxy_url.as_str())).unwrap();
        // dbnode_msgcache
        //     .save_msg(
        //         &_msg.id,
        //         format!(
        //             "{}{}\n> ---MSG_TYPE--- {} `||` At: {}",
        //             &_msg.content,
        //             &_attachments,
        //             &_msg.author,
        //             &_msg.timestamp.format("%H:%M:%S %p")
        //         ),
        //     )
        //     .await;

    }

    Ok(())

    //
    // Auto respond on keywords
    //

    // let dbnode_notes = Database::from("notes".to_string()).await;
    // let ref_msg = &_msg.referenced_message;

    // let options = MatchOptions {
    //     case_sensitive: false,
    //     require_literal_separator: false,
    //     require_literal_leading_dot: false,
    // };
    // if !_msg.author.bot && !_msg.content.contains("dnote ") {
    //     for entry in glob_with(format!("{}/*", dbnode_notes).as_str(), options).unwrap() {
    //         match entry {
    //             Ok(path) => {
    //                 let note = path.file_name().unwrap().to_string_lossy().to_string();

    //                 if _msg
    //                     .content
    //                     .to_lowercase()
    //                     .contains(&note.as_str().to_lowercase())
    //                 {
    //                     let typing = _ctx
    //                         .http
    //                         .start_typing(u64::try_from(_msg.channel_id).unwrap())
    //                         .unwrap();

    //                     // Use contentsafe options
    //                     let settings = {
    //                         ContentSafeOptions::default()
    //                             .clean_channel(false)
    //                             .clean_role(true)
    //                             .clean_user(false)
    //                             .clean_everyone(true)
    //                             .clean_here(true)
    //                     };

    //                     let content = content_safe(
    //                         &_ctx.cache,
    //                         Note::from(&note).await.get_contents().await,
    //                         &settings,
    //                     )
    //                     .await;
    //                     if ref_msg.is_some() {
    //                         ref_msg
    //                             .as_ref()
    //                             .map(|x| x.reply_ping(&_ctx.http, &content))
    //                             .unwrap()
    //                             .await
    //                             .unwrap()
    //                             .react(&_ctx.http, '❎')
    //                             .await
    //                             .unwrap();
    //                     } else {
    //                         _msg.reply(&_ctx.http, &content)
    //                             .await
    //                             .unwrap()
    //                             .react(&_ctx.http, '❎')
    //                             .await
    //                             .unwrap();
    //                     }
    //                     typing.stop();
    //                 }
    //             }
    //             Err(e) => println!("{:?}", e),
    //         }
    //     }
    // }

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

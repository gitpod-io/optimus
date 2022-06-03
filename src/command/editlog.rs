use super::*;

#[command]
pub async fn editlog(_ctx: &Context, _msg: &Message) -> CommandResult {
    let ref_msg = &_msg.referenced_message;
    // Use contentsafe options
    let settings = { ContentSafeOptions::default().clean_user(true) };
    if ref_msg.is_some() {
        if ref_msg
            .as_ref()
            .map(|x| x.edited_timestamp.is_some())
            .unwrap()
        {
            let ref_msg_id = ref_msg.as_ref().unwrap().id;
            let dbnode = Database::from("msgcache".to_string()).await;
            let content = content_safe(
                &_ctx.cache,
                dbnode
                    .fetch_msg(ref_msg_id)
                    .await
                    .replace("---MSG_TYPE---", "Edited:"),
                &settings,
                &[],
            );

            ref_msg
                .as_ref()
                .map(|x| x.reply(&_ctx.http, &content))
                .unwrap()
                .await?;
            // _msg.reply(&_ctx.http, &content).await?;
        } else {
            _msg.reply(&_ctx.http, "Not an edited message").await?;
        }
    } else {
        _msg.reply(
            &_ctx.http,
            "Use this command while replying to an edited message",
        )
        .await?;
    }

    Ok(())
}

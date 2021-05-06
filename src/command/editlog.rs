use super::*;

#[command]
pub async fn editlog(_ctx: &Context, _msg: &Message) -> CommandResult {
    let ref_msg = &_msg.referenced_message;
    let ref_msg_id = ref_msg.as_ref().unwrap().id;
    let dbnode = Database::from("msgcache".to_string()).await;
    let content = dbnode.fetch_msg(ref_msg_id).await.replace("---MSG_TYPE---", "Edited:");

    if ref_msg.is_some() {
        _msg.reply(&_ctx.http, &content).await?;
    } else {
        _msg.reply(
            &_ctx.http,
            "Use this command while replying to an edited message",
        )
        .await?;
    }

    Ok(())
}

use super::*;

pub async fn responder(
    _ctx: Context,
    _old_if_available: Option<Message>,
    _new: Option<Message>,
    _event: MessageUpdateEvent,
) {
    // let last_msg_id = _new
    //     .unwrap()
    //     .channel(&_ctx.cache)
    //     .await
    //     .unwrap()
    //     .guild()
    //     .unwrap()
    //     .last_message_id
    //     .unwrap();

    // let dbnode = Database::from("msgcache".to_string()).await;

    // let map = json!({"content": dbnode.fetch_deleted_msg(_event.id).await.replace("---MSG_TYPE---", "Edited:")});
    // _ctx.http
    //     .send_message(u64::try_from(_event.channel_id).unwrap(), &map)
    //     .await
    //     .ok();

    let _msg_id = _event.id;
    let _channel_id = _event.channel_id;

    if let Ok(message) = &_ctx
        .http
        .get_message(
            u64::try_from(_channel_id).unwrap(),
            u64::try_from(_msg_id).unwrap(),
        )
        .await
    {
        if message.edited_timestamp.is_some() && message.webhook_id.is_none() {
            let dbnode = Database::from("msgcache".to_string()).await;
            let msg_content = &message.content;

            // let mut is_self_reacted = false;
            // for user in message.reactions.iter() {
            //     if user.me {
            //         is_self_reacted = true;
            //     }
            // }

            // if !is_self_reacted && !message.is_own(&_ctx.cache).await {
            //     message.react(&_ctx.http, '✍').await.ok();
            // }
            let edit_time = &message.edited_timestamp.unwrap().format("%H:%M:%S %p");
            let old_content = dbnode.fetch_msg(_msg_id).await;
            let new_content = format!(
                "{}\n> Edited at: {}\n{}",
                &msg_content, &edit_time, &old_content
            );
            dbnode.save_msg(&_msg_id, new_content).await;
            // message.delete_reaction_emoji(&_ctx.http, '✍').await.unwrap();
        }
    }
}

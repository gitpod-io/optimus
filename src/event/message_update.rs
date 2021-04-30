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

    // let map = json!({"content": dbnode.fetch_deleted_msg(_event.id).await.replace("~~MSG_TYPE~~", "Edited:")});
    // _ctx.http
    //     .send_message(u64::try_from(_event.channel_id).unwrap(), &map)
    //     .await
    //     .ok();

    let _msg_id = _event.id;
    let _channel_id = _event.channel_id;

    let message = &_ctx
        .http
        .get_message(
            u64::try_from(_channel_id).unwrap(),
            u64::try_from(_msg_id).unwrap(),
        )
        .await
        .unwrap();

    if message.edited_timestamp.is_some() {
        message.react(&_ctx.http, '✍').await.unwrap();
    }
}

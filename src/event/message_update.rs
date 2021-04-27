use super::*;

pub async fn responder(
    _ctx: Context,
    _old_if_available: Option<Message>,
    _new: Option<Message>,
    _event: MessageUpdateEvent,
) {
    let map = json!({"content": fetch_msgcache_by_id(_event.id).await.replace("~~MSG_TYPE~~", "Edited:")});
    _ctx.http
        .send_message(u64::try_from(_event.channel_id).unwrap(), &map)
        .await
        .ok();
}

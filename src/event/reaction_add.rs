use super::*;

pub async fn responder(_ctx: Context, _added_reaction: Reaction) {
    let emoji = &_added_reaction.emoji.to_string();
    let is_self_msg = &_added_reaction
        .message(&_ctx.http)
        .await
        .unwrap()
        .is_own(&_ctx.cache)
        .await;
    let is_bot = &_added_reaction.user(&_ctx.http).await.unwrap().bot;

    let react_data = &_added_reaction
        .message(&_ctx.http)
        .await
        .unwrap();


     let is_self_reacted = react_data.reactions
        .iter()
        .as_ref()
        .first()
        .unwrap()
        .me;

    if !*is_bot && is_self_reacted && *is_self_msg && *emoji == String::from('❎') {
        _added_reaction
            .message(&_ctx.http)
            .await
            .unwrap()
            .delete(&_ctx.http)
            .await
            .unwrap();
    }
}

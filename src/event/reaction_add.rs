use super::*;

pub async fn responder(_ctx: Context, _added_reaction: Reaction) {
    let emoji = &_added_reaction.emoji.to_string();
    let a_bot_reacted_now = &_added_reaction.user(&_ctx.http).await.unwrap().bot;

    let react_data = &_added_reaction.message(&_ctx.http).await.unwrap();
    let is_self_msg = react_data.is_own(&_ctx.cache).await;
    // let reactions_count = react_data.reactions.iter().count();
    let is_self_reacted = react_data.reactions.iter().as_ref().first().unwrap().me;

    let reactated_user = &_added_reaction.user(&_ctx.http).await.unwrap();

    match emoji.as_str() {
        "✍" => {
            if !*a_bot_reacted_now && is_self_reacted {
                react_data
                    .delete_reaction_emoji(&_ctx.http, '✍')
                    .await
                    .unwrap();

                let dbnode = Database::from("msgcache".to_string()).await;
                // Use contentsafe options
                let settings = {
                    ContentSafeOptions::default()
                        .clean_channel(false)
                        .clean_role(true)
                        .clean_user(false)
                        .clean_everyone(true)
                        .clean_here(true)
                };

                let content = content_safe(
                    &_ctx.cache,
                    dbnode.fetch_deleted_msg(_added_reaction.message_id).await,
                    &settings,
                )
                .await;

                react_data
                    .reply(
                        &_ctx.http,
                        content.replace(
                            "~~MSG_TYPE~~",
                            format!("Asked by {}  ||  Edited by", &reactated_user).as_str(),
                        ),
                    )
                    .await
                    .unwrap();
            }
        }
        "❎" => {
            if !*a_bot_reacted_now && is_self_reacted && is_self_msg {
                react_data.delete(&_ctx.http).await.unwrap();
            }
        }
        _ => {}
    }

    // if !*is_bot && is_self_reacted && *is_self_msg && *emoji == String::from('❎') {
    //     _added_reaction
    //         .message(&_ctx.http)
    //         .await
    //         .unwrap()
    //         .delete(&_ctx.http)
    //         .await
    //         .unwrap();
    // }
}

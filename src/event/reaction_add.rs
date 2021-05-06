use substr::StringUtils;

use super::*;

pub async fn responder(_ctx: Context, _added_reaction: Reaction) {
    let emoji = &_added_reaction.emoji.to_string();
    let reacted_user = &_added_reaction.user(&_ctx.http).await.unwrap();
    let a_bot_reacted_now = &reacted_user.bot;

    let react_data = &_added_reaction.message(&_ctx.http).await.unwrap();
    let is_self_msg = react_data.is_own(&_ctx.cache).await;
    // let reactions_count = react_data.reactions.iter().count();
    let reactions = &react_data.reactions;

    let mut is_self_reacted = false;
    for user in reactions.iter() {
        if user.me {
            is_self_reacted = true;
        }
    }

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
                        .clean_user(true)
                        .clean_everyone(true)
                        .clean_here(true)
                };

                let content = content_safe(
                    &_ctx.cache,
                    dbnode.fetch_msg(_added_reaction.message_id).await,
                    &settings,
                )
                .await;

                react_data
                    .reply(
                        &_ctx.http,
                        content
                            .replace(
                                "---MSG_TYPE---",
                                format!("Triggered: {} `||` Edited:", &reacted_user).as_str(),
                            )
                            .as_str()
                            .substring(0, 2000),
                    )
                    .await
                    .unwrap();

                // let msg_content = &react_data.content;
                // let edit_time = &react_data.edited_timestamp.unwrap().format("%H:%M:%S %p");
                // let old_content = dbnode.fetch_msg(react_data.id).await;
                // let new_content = format!(
                //     "{}\n> Edited at: {}\n {}",
                //     &msg_content, &edit_time, &old_content
                // );
                // dbnode.save_msg(&react_data.id, new_content).await;
            }
        }

        "📩" => {
            if !*a_bot_reacted_now && is_self_reacted {
                react_data
                    .delete_reaction_emoji(&_ctx.http, '📩')
                    .await
                    .unwrap();

                let dbnode = Database::from("delmsg_trigger".to_string()).await;

                let content = dbnode.fetch_msg(_added_reaction.message_id).await.replace(
                    "---MSG_TYPE---",
                    format!("Triggered: {} `||` Deleted:", &reacted_user).as_str(),
                );

                react_data
                    .reply(&_ctx.http, content.as_str().substring(0, 2000))
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

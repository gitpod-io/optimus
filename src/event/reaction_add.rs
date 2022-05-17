use serenity::model::Permissions;
use substr::StringUtils;

use super::*;

pub async fn responder(_ctx: Context, _added_reaction: Reaction) {
    let emoji = &_added_reaction.emoji.to_string();
    let reacted_user = &_added_reaction.user(&_ctx.http).await.unwrap();
    if reacted_user.bot {
        return;
    }
    let react_data = &_added_reaction.message(&_ctx.http).await.unwrap();

    let is_self_msg = &react_data.is_own(&_ctx.cache).await;
    // let reactions_count = react_data.reactions.iter().count();
    let reactions = &react_data.reactions;

    let mut is_self_reacted = false;
    for user in reactions.iter() {
        if user.me {
            is_self_reacted = true;
        }
    }

    match emoji.as_str() {
        // "✍" => {
        //     if !*a_bot_reacted_now && is_self_reacted {
        //         react_data
        //             .delete_reaction_emoji(&_ctx.http, '✍')
        //             .await
        //             .unwrap();

        //         let dbnode = Database::from("msgcache".to_string()).await;
        //         // Use contentsafe options
        //         let settings = {
        //             ContentSafeOptions::default()
        //                 .clean_channel(false)
        //                 .clean_role(true)
        //                 .clean_user(true)
        //                 .clean_everyone(true)
        //                 .clean_here(true)
        //         };

        //         let content = content_safe(
        //             &_ctx.cache,
        //             dbnode.fetch_msg(_added_reaction.message_id).await,
        //             &settings,
        //         )
        //         .await;

        //         react_data
        //             .reply(
        //                 &_ctx.http,
        //                 &content
        //                     .replace(
        //                         "---MSG_TYPE---",
        //                         format!("Triggered: {} `||` Edited:", &reacted_user).as_str(),
        //                     )
        //                     .as_str()
        //                     .substring(0, 2000),
        //             )
        //             .await
        //             .unwrap()
        //             .react(&_ctx.http, '🔃')
        //             .await
        //             .unwrap();

        //         // let msg_content = &react_data.content;
        //         // let edit_time = &react_data.edited_timestamp.unwrap().format("%H:%M:%S %p");
        //         // let old_content = dbnode.fetch_msg(react_data.id).await;
        //         // let new_content = format!(
        //         //     "{}\n> Edited at: {}\n {}",
        //         //     &msg_content, &edit_time, &old_content
        //         // );
        //         // dbnode.save_msg(&react_data.id, new_content).await;
        //     }
        // }

        // Deleted message handlers and or listeners
        "📩" => {
            if is_self_reacted {
                let roles = &_added_reaction.member.unwrap().roles;
                let is_owner = _added_reaction
                    .guild_id
                    .unwrap()
                    .to_partial_guild(&_ctx)
                    .await
                    .unwrap()
                    .owner_id
                    == reacted_user.id;
                let mut got_admin = false;

                for role in roles {
                    if role
                        .to_role_cached(&_ctx.cache)
                        .await
                        .map_or(false, |r| r.has_permission(Permissions::ADMINISTRATOR))
                    {
                        got_admin = true;
                        break;
                    }
                }
                if got_admin || is_owner {
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
                        .reply(&_ctx.http, &content.as_str().substring(0, 2000))
                        .await
                        .unwrap()
                        .react(&_ctx.http, '🔃')
                        .await
                        .unwrap();
                }
            }
        }

        "🔃" => {
            if *is_self_msg && is_self_reacted {
                react_data
                    .delete_reaction_emoji(&_ctx.http, '🔃')
                    .await
                    .unwrap();
                react_data.delete(&_ctx.http).await.unwrap();

                let target_emoji = {
                    if react_data.content.to_string().contains("`||` Edited: ") {
                        '✍'
                    } else {
                        '📩'
                    }
                };

                react_data
                    .referenced_message
                    .as_ref()
                    .map(|x| async move {
                        x.react(&_ctx.http, target_emoji).await.unwrap();
                    })
                    .unwrap()
                    .await;
            }
        }

        "❎" => {
            if is_self_reacted && *is_self_msg {
                react_data.delete(&_ctx.http).await.unwrap();
            }
        }
        _ => {}
    }
}

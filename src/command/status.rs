use super::*;
use serenity::model::gateway::ClientStatus;
use serenity::model::prelude::ActivityType;
// use serenity::model::user::OnlineStatus;

// async fn get_online_status(input: &OnlineStatus) -> String {
//     let mut mode = String::new();

//     match input {
//         OnlineStatus::DoNotDisturb => {
//             mode.push_str("Dnd");
//         }
//         OnlineStatus::Idle => {
//             mode.push_str("Idle");
//         }
//         OnlineStatus::Invisible => {
//             mode.push_str("Invisible");
//         }
//         OnlineStatus::Offline => {
//             mode.push_str("Offline");
//         }
//         _ => {}
//     }
//     mode
// }

#[command]
#[only_in(guilds)]
#[description = "Pull the status of an user"]
pub async fn status(_ctx: &Context, _msg: &Message, mut _args: Args) -> CommandResult {
    let user = Parse::user(&_ctx,&_msg, &_args).await;
    let guild_id = &_msg.guild_id.unwrap();
    let user_data = &_ctx
        .http
        .get_member(*guild_id.as_u64(), user)
        .await
        .unwrap();

    let user_status = _ctx
        .cache
        .guild(*guild_id.as_u64())
        .await
        .unwrap()
        .presences;

    let mut status_content = String::new();
    let mut status_type = String::new();
    let mut status_client = String::new();
    // let mut status_mode = String::new();

    for (user_id, presence) in user_status {
        if *user_id.as_u64() == user {
            let one = presence.activities;

            let client_data = presence.client_status.unwrap();

            match client_data {
                ClientStatus {
                    desktop,
                    mobile,
                    web,
                } => {
                    if desktop.is_some() {
                        status_client.push_str("Desktop");
                        // status_mode.push_str(get_online_status(&desktop.unwrap()).await.as_str());
                    } else if mobile.is_some() {
                        status_client.push_str("Mobile");
                        // status_mode.push_str(get_online_status(&mobile.unwrap()).await.as_str());
                    } else if web.is_some() {
                        status_client.push_str("Web");
                        // status_mode.push_str(get_online_status(&web.unwrap()).await.as_str());
                    }
                } // _ => {}
            }

            for acti in one {
                // status.push_str(&acti.emoji.unwrap().);

                status_type.push_str(&acti.name);

                match acti.kind {
                    ActivityType::Custom => {
                        status_content.push_str(&acti.state.unwrap());
                    }

                    _ => {
                        status_content.push_str("None");
                    }
                }
            }

            break;
        }
    }

    _msg.channel_id
        .send_message(&_ctx.http, |m| {
            m.embed(|e| {
                e.title(format!("**{}**'s status", &user_data.display_name()));

                // e.field("Mode", &status_mode, false);

                if !&status_type.is_empty() {
                    e.field("Type", &status_type, false);
                } else {
                    e.field("Respose", "Not set or offline", false);
                }

                if !&status_client.is_empty() {
                    e.field("Using from", &status_client, false);
                }

                if !&status_content.is_empty() {
                    e.field("Content", &status_content, false);
                }

                e
            });

            m
        })
        .await
        .unwrap();

    Ok(())
}

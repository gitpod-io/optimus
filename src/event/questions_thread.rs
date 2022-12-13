use super::*;
use crate::db::ClientContextExt;
use serenity::{futures::StreamExt, utils::MessageBuilder};

pub async fn responder(_ctx: &Context) {
    // #questions, #selfhosted-questions, #openvscode-questions, #documentation
    let db = _ctx.get_db().await;
    let channels = db.get_question_channels().await.unwrap();

    for channel_id in channels {
        let channel_id = ChannelId(*channel_id.id.as_u64());

        let last_msg_id = _ctx
            .http
            .get_messages(*channel_id.as_u64(), "")
            .await
            .unwrap();

        let last_msg_id = last_msg_id.first();

        if last_msg_id.is_some() && last_msg_id.unwrap().is_own(&_ctx.cache) {
            continue;
        }

        // Might need to do this in the future for race conditions
        // let last_msg_id2 = _ctx
        //     .http
        //     .get_channel(*channel_id.as_u64())
        //     .await
        //     .unwrap()
        //     .guild()
        //     .unwrap()
        //     .last_message_id
        //     .unwrap();
        // let qq = _ctx.http.get_messages(*channel_id.as_u64(), "").await;

        // // Clean out any leftover placeholders for thread-help upto 3 messages
        // if qq.is_ok() {
        //     let mut _count = 0;
        //     for message in qq.as_ref().unwrap().iter() {
        //         if _count > 3 {
        //             break;
        //         }
        //         if message.is_own(&_ctx).await {
        //             message.delete(&_ctx).await.unwrap();
        //         }
        //         _count += 1;
        //     }
        // }

        // // Place the placeholder
        // let msg = qq.unwrap();
        // let last_msg = msg.first().unwrap();
    }

    let placeholder_text = "**Press the button below** 👇 to gain access to the server";

    let mut t = GETTING_STARTED_CHANNEL.messages_iter(&_ctx.http).boxed();
    while let Some(message_result) = t.next().await {
        match message_result {
            Ok(message) => {
                if message.content == *placeholder_text {
                    return;
                }
            }
            Err(error) => {
                eprintln!("Uh oh! Error: {}", error);
                return;
            }
        }
    }

    GETTING_STARTED_CHANNEL
        .send_message(&_ctx.http, |m| {
            m.content(&placeholder_text);
            m.components(|c| {
                c.create_action_row(|a| {
                    a.create_button(|b| {
                        b.label("Let's go")
                            .custom_id("getting_started_letsgo")
                            .style(ButtonStyle::Primary)
                            .emoji(ReactionType::Unicode("🙌".to_string()))
                    })
                })
            });
            m
        })
        .await
        .unwrap();
}

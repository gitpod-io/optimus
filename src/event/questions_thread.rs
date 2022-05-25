use serenity::utils::MessageBuilder;

use crate::db::ClientContextExt;

use super::*;

pub async fn responder(_ctx: &Context) {
    // #questions, #selfhosted-questions, #openvscode-questions, #documentation
    let db = _ctx.get_db().await;
    let channels = db.get_question_channels().await.unwrap();

    for channel_id in channels {
        let channel_id = ChannelId(*channel_id.id.as_u64());

        // Might need to do this in the future for race conditions
        let last_msg_id = _ctx
            .http
            .get_messages(*channel_id.as_u64(), "")
            .await
            .unwrap();

        let last_msg_id = last_msg_id.first();

        if last_msg_id.is_some() && last_msg_id.unwrap().is_own(&_ctx.cache).await {
            continue;
        }
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

        let _m = channel_id
			.send_message(&_ctx, |m| {
				m.content(
					MessageBuilder::new()
						.push_underline_line("**Welcome to the Gitpod community!**")
						.push_line("Community is at the heart of Gitpod, we’re happy to help you out 🧡")
						.push_line("Before **asking a question**, remember to check out our documentation or watch our screencasts.")
						.push_line("If you think Gitpod is not working, please check our status page. Thank you!").build());
				m.components(|c| {
					c.create_action_row(|ar| {
						ar.create_button(|button| {
							button
								.style(ButtonStyle::Primary)
								.label("Ask a question")
								.custom_id("gitpod_create_issue")
								.emoji(ReactionType::Unicode("💡".to_string()))
						});
						ar.create_button(|button| {
							button
								// .custom_id("gitpod_docs_link")
								.style(ButtonStyle::Link)
								.label("Docs")
								.emoji(ReactionType::Unicode("📚".to_string()))
								.url("https://www.gitpod.io/docs/")
						});
						ar.create_button(|button| {
							button.style(ButtonStyle::Link).label("YouTube").url(
								"https://youtube.com/playlist?list=PL3TSF5whlprXVp-7Br2oKwQgU4bji1S7H",
							).emoji(ReactionType::Unicode("📺".to_string()))
						});
						ar.create_button(|button| {
							button
								.style(ButtonStyle::Link)
								.label("Status")
								.emoji(ReactionType::Unicode("🧭".to_string()))
								.url("https://www.gitpodstatus.com/")
						})
					})
				})
			})
			.await
			.unwrap();
    }
}

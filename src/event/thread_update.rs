use super::*;

// Was trying to hook into auto thread archival and ask the participants
// if the thread was resolved but guess we can't reliably do it now
// since there is no reliable way to detect who triggered thread_update
pub async fn responder(_ctx: Context, _thread: GuildChannel) {
    let thread_type = {
        if _thread.name.contains("✅") || _thread.name.contains("❓") {
            "question"
        } else {
            "thread"
        }
    };

    let last = _thread.last_message_id.unwrap();
    let is_self = &_ctx
        .http
        .get_message(*_thread.id.as_u64(), *last.as_u64())
        .await
        .unwrap();

    if !is_self.is_own(&_ctx.cache).await && _thread.thread_metadata.unwrap().archived {
        _thread
                .send_message(&_ctx, |m| {
                    m.content(format!("> This {} did not have any recent activity or wasn't normally closed. Feel free to `/close` it or post an update if anything is unresolved.", thread_type)).components(|c| {
                        c.create_action_row(|ar| {
                            ar.create_button(|button| {
                                button
                                    .style(ButtonStyle::Success)
                                    .label("Close")
                                    .custom_id("gitpod_close_issue")
                                    .emoji(ReactionType::Unicode("✉️".to_string()))
                            })
                        })
                    })
                })
                .await
                .unwrap();
        // let tp = _thread.id.get_thread_members(&_ctx.http).await.unwrap();

        // for mem in tp.iter() {
        //     dbg!(mem.user_id);
        // }

        // let stuff_str: String = tp
        //     .into_iter()
        //     .map(|i| i.mention().to_string())
        //     .collect::<String>();
        // println!("{}", stuff_str);

        // _thread
        //     .edit_thread(&_ctx, |t| t.archived(true))
        //     .await
        //     .unwrap();
    } else if is_self.is_own(&_ctx.cache).await && _thread.thread_metadata.unwrap().archived {
        _thread
            .edit_thread(&_ctx.http, |t| t.archived(false))
            .await
            .unwrap();
    }
}

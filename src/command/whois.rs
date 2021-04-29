use super::*;

#[command]
// #[sub_commands(fetch)]
pub async fn whois(_ctx: &Context, _msg: &Message) -> CommandResult {
    {
        let typing = _ctx
            .http
            .start_typing(u64::try_from(_msg.channel_id).unwrap())
            .unwrap();

        let dbnode = Database::from("userid".to_string()).await;
        let user = &_args.rest().to_string().replace("<@!", "").replace(">", "");
        let guid = &_msg.guild_id.unwrap();

        let prev_usernames = dbnode
            .get_user_info(&user.trim_matches(' ').to_string())
            .await;

        let user_date = &_msg.author.created_at().date().format("%a, %B %e, %Y");
        let user_data = &_ctx
            .cache
            .guild(guid)
            .await
            .unwrap()
            .member(&_ctx.http, user.parse::<u64>().unwrap())
            .await
            .unwrap();

        let user_server_date = &user_data.joined_at.unwrap().date().format("%a, %B %e, %Y");

        let user_time = &_msg.author.created_at().time().format("%H:%M:%S %p");

        let user_avatar = &user_data.user.avatar_url().unwrap();

        let user_colors = &user_data.colour(&_ctx.cache).await.unwrap().hex();

        // let guild = &_msg.guild_id.unwrap();
        // let messages = &_ctx.cache.guild_channels(guild).await.unwrap();

        // let mut countmsg = 1;

        // for (id, channel) in messages.iter() {
        //     countmsg = countmsg + 1;
        //     let chi = id.messages_iter(&_ctx.http).un;
        // }

        // let hmm = &_msg.channel(&_ctx.cache).await.unwrap().guild().unwrap().messages(http, builder)

        _msg.channel_id
            .send_message(&_ctx.http, |m| {
                m.embed(|e| {
                    e.title("FreeBSD whois");
                    // e.color()
                    e.thumbnail(user_avatar);

                    e.field("Intoduction.exe", format!("A {}", english_gen(2, 1)), true);
                    e.field(
                        "RegisteredAt.exe",
                        format!("{} {}", &user_date, &user_time),
                        true,
                    );

                    e.field("‎", "‎", false);

                    e.field("JoinedAt.exe", &user_server_date, true);
                    e.field("Color.exe", format!("#{} (Heximal)", &user_colors), true);

                    e.field(
                        "Usernames.exe:",
                        format!(
                            "{}{}{}",
                            "```\n",
                            prev_usernames.as_str().substring(0, 1016),
                            "```\n"
                        ),
                        false,
                    );

                    e
                });

                m
            })
            .await
            .unwrap();
        typing.stop();
    }
    Ok(())
}

// #[command]
// #[required_permissions(ADMINISTRATOR)]
// pub async fn fetch(_ctx: &Context, _msg: &Message) -> CommandResult {
//     {
//         let typing = _ctx
//             .http
//             .start_typing(u64::try_from(_msg.channel_id).unwrap())
//             .unwrap();
//         let dbnode_userid = Database::from("userid".to_string()).await;
//         let members = &_msg.guild(&_ctx.cache).await.unwrap().members;

//         for (_user_id, _member) in members {
//             dbnode_userid
//                 .save_user_info(&_user_id, _member.user.tag())
//                 .await;
//         }

//         typing.stop();
//     }
//     Ok(())
// }

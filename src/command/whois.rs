use super::*;

#[command]
#[aliases("whoami")]
// #[sub_commands(fetch)]
pub async fn whois(_ctx: &Context, _msg: &Message, mut _args: Args) -> CommandResult {
    {
        let typing = _ctx
            .http
            .start_typing(u64::try_from(_msg.channel_id).unwrap())
            .unwrap();

        let dbnode = Database::from("userid".to_string()).await;

        let user = Parse::user(&_msg, &_args);

        let guid = &_msg.guild_id.unwrap();

        let prev_usernames = dbnode.get_user_info(&user.to_string()).await;

        let user_data = &_ctx
            .cache
            .guild(guid)
            .await
            .unwrap()
            .member(&_ctx.http, user)
            .await
            .unwrap();

        let user_date = &user_data.user.created_at().date().format("%a, %B %e, %Y");

        let user_server_date = &user_data.joined_at.unwrap().date().format("%a, %B %e, %Y");

        let user_time = &_msg.author.created_at().time().format("%H:%M:%S");

        let user_avatar = &user_data.user.face();

        let user_colors = {
            let user = &user_data.colour(&_ctx.cache).await;
            if user.is_some() {
                user.unwrap().hex()
            } else {
                String::from("0000")
            }
        };

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
                    e.thumbnail(&user_avatar);
                    e.url("https://www.freebsd.org/cgi/man.cgi?query=whois");

                    let intro = english_gen(1, 1);

                    e.fields(vec![
                        (
                            "Intoduction.exe",
                            format!(
                                "{} [{}](https://www.dictionary.com/browse/{})",
                                vowel_gen(&intro),
                                &intro,
                                &intro
                            ),
                            true,
                        ),
                        (
                            "Color.exe",
                            format!(
                                "[#{}](https://www.colorhexa.com/{}) (Heximal)",
                                &user_colors, &user_colors
                            ),
                            true,
                        ),
                    ]);

                    e.field("‎", "‎", false);

                    e.fields(vec![
                        ("JoinedAt.exe", format!("{}", &user_server_date), true),
                        (
                            "RegisteredAt.exe",
                            format!("{} {}", &user_date, &user_time),
                            true,
                        ),
                    ]);

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

                    // e.image(&user_avatar);

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

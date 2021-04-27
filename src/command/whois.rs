use super::*;

#[command]
pub async fn whois(_ctx: &Context, _msg: &Message) -> CommandResult {
    {
        let typing = _ctx
            .http
            .start_typing(u64::try_from(_msg.channel_id).unwrap())
            .unwrap();
        let dbnode = Database::from("userid".to_string()).await;
        let user = &_args.rest().to_string().replace("<@!", "").replace(">", "");

        let prev_usernames = dbnode
            .get_user_info(&user.trim_matches(' ').to_string())
            .await;

        _msg.channel_id
            .send_message(&_ctx.http, |m| {
                m.embed(|e| {
                    e.title("FreeBSD whois");
                    e.description(format!("A {} {}", english_gen(1, 2), &_args.rest()));
                    e.field(
                        "Previous usernames:",
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

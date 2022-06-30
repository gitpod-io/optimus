use super::*;

pub async fn responder(
    _ctx: Context,
    _guild_id: GuildId,
    _user: User,
    _member_data_if_available: Option<Member>,
) {
    let _system_channel_id = _ctx
        .cache
        .guild(&_guild_id)
        .map(|x| x.system_channel_id)
        .unwrap()
        .unwrap();

    _ctx.http
        .send_message(
            u64::try_from(_system_channel_id).unwrap(),
            &json!({
                "content":
                    format!(
                        "> :arrow_forward: **{}** (**{}**) is no more <a:duckdance:835457840365568012>, sed lyf...",
                        _user.tag(),
                        _user.id
                    )
            }),
        )
        .await
        .unwrap();
}

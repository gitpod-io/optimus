use super::*;

pub async fn responder(_ctx: Context, _guild_id: GuildId, _new_member: Member) {
    let user_date = _new_member.user.created_at().naive_utc().date();
    let user_time = _new_member.user.created_at().naive_utc().time();
    let _system_channel_id = _ctx
        .cache
        .guild(&_guild_id)
        .await
        .map(|x| x.system_channel_id)
        .unwrap()
        .unwrap();

    _ctx.http
        .send_message(
            u64::try_from(_system_channel_id).unwrap(),
            &json!({
                "content":
                    format!(
                        "> :arrow_forward: {}'s account Date: **{}**; Time: **{}**",
                        _new_member.display_name(),
                        &user_date,
                        &user_time
                    )
            }),
        )
        .await
        .unwrap();
}

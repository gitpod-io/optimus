use super::*;

pub async fn responder(_ctx: Context, _guild_id: GuildId, _new_member: Member) {
    let user_date = _new_member.user.created_at().date().format("%a, %B %e, %Y");
    let user_time = _new_member.user.created_at().time().format("%H:%M:%S %p");
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
                        "> :arrow_forward: {}'s reg Date: **{}**; Time: **{}**",
                        _new_member.display_name(),
                        &user_date,
                        &user_time
                    )
            }),
        )
        .await
        .unwrap();

    let jailbreak_channel = _ctx
        .cache
        .guild(_guild_id)
        .await
        .unwrap()
        .channel_id_from_name(&_ctx.cache, "jailbreak")
        .await;

    if jailbreak_channel.is_some() {
        jailbreak_channel
            .unwrap()
            .send_message(&_ctx.http, |x| x.content(format!("> {} welcome to our server, you will be given full access if your account is verified legitimate after the verification process is done, meanwhile you can ask about anything at here.", _new_member.mention())))
            .await
            .unwrap();
    }

    let blacklist = fs::read_to_string(format!(
        "{}/db/blacklisted_names",
        env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_string_lossy()
    ))
    .await
    .unwrap();

    if blacklist.contains(&_new_member.display_name().to_ascii_uppercase()) {
        _new_member
            .user
            .direct_message(&_ctx.http, |m| m.content("Lacks a Brain"))
            .await
            .unwrap();
        _new_member
            .ban_with_reason(&_ctx.http, 0, "Missing Brain.exe")
            .await
            .unwrap();
    }
}

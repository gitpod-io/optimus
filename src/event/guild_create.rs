// This event is intended to be dispatched when our bot joins a new discord server.;
// Although thats not the only thing this event is for.

use super::*;

async fn welcome_msg(_ctx: &Context, channel: &ChannelId, guild: &Guild) {
    &_ctx
        .http
        .send_message(
            *channel.as_u64(),
            &json!({
                "content": format!("Optimus at your service to robotify **{}**!", &guild.name)
            }),
        )
        .await
        .unwrap();
}

pub async fn responder(_ctx: Context, _guild: Guild, _is_new: bool) {
    if _is_new {
        // At first log in base server
        _ctx.http
            .send_message(
                842668777363865610,
                &json!({
                    "content": format!("I was invited to **{}** (`{}`)", &_guild.name, &_guild.id)
                }),
            )
            .await
            .unwrap();

        // Then do a welcome message at the new server
        let _new_guild_syschan_id = &_guild.system_channel_id;
        if _new_guild_syschan_id.is_some() {
            welcome_msg(&_ctx, &_new_guild_syschan_id.unwrap(), &_guild).await;
        } else {
            for (_channel_id, _guild_channel) in &_guild.channels(&_ctx.http).await.unwrap() {
                let _msgs = &_ctx.http.get_messages(*_channel_id.as_u64(), "").await;

                if _msgs.is_ok() {
                    if _msgs.as_ref().unwrap().iter().count() > 200 {
                        welcome_msg(&_ctx, &_channel_id, &_guild).await;
                        break;
                    }
                }
            }
        }
    }
}

use super::*;

#[command]
pub async fn av(_ctx: &Context, _msg: &Message, mut _args: Args) -> CommandResult {
    let user = Parse::user(&_ctx, &_msg, &_args).await;
    let guild_id = &_msg.guild_id.unwrap();
    let user_data = &_ctx
        .http
        .get_member(*guild_id.as_u64(), user)
        .await
        .unwrap();

    _msg.channel_id
        .send_message(&_ctx.http, |m| {
            m.embed(|e| {
                e.title(format!("**{}**", &user_data.display_name()));
                e.url(format!(
                    "https://images.google.com/searchbyimage?image_url={}",
                    &user_data.user.face()
                ));
                e.image(&user_data.user.face());

                e
            });

            m
        })
        .await
        .unwrap();

    Ok(())
}

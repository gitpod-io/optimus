use super::*;

#[command]
pub async fn invite(_ctx: &Context, _msg: &Message) -> CommandResult {
    _msg.channel_id
        .send_message(&_ctx.http, |m| {
            m.embed(|e| {
                e.title("Robotify your server with optimus!");
                e.field("Invite me", format!("[Click here]({}) to do so.", "https://discord.com/oauth2/authorize?client_id=648118759105757185&scope=bot&permissions=8"), false);
                e
            });

            m
        })
        .await?;

    Ok(())
}

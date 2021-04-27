use super::*;

#[command]
pub async fn about(_ctx: &Context, _msg: &Message) -> CommandResult {
    _msg.reply(&_ctx.http, "Welp! Just another noobish *thing*.")
        .await?;

    Ok(())
}

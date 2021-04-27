use super::*;

#[command]
async fn some_long_command(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.channel_id
        .say(&ctx.http, &format!("Arguments: {:?}", args.rest()))
        .await?;

    Ok(())
}

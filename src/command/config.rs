use super::*;

// A command can have sub-commands, just like in command lines tools.
// Imagine `cargo help` and `cargo help run`.
#[command("config")]
#[sub_commands(
    questions_channel,
    introduction_channel,
    subscriber_role,
    getting_started
)]
async fn config(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    if !_args.is_empty() {
        msg.reply(&ctx.http, format!("{} is not a config", _args.rest()))
            .await?;
    }
    Ok(())
}

// This will only be called if preceded by the `upper`-command.
#[command]
// #[aliases("sub-command", "secret")]
#[description("Set the question channels to watch for")]
async fn questions_channel(_ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    Ok(())
}
#[command]
// #[aliases("sub-command", "secret")]
#[description("This is `upper`'s sub-command.")]
async fn introduction_channel(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    msg.reply(&ctx.http, "This is a sub function!").await?;

    Ok(())
}
#[command]
// #[aliases("sub-command", "secret")]
#[description("This is `upper`'s sub-command.")]
async fn getting_started(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    msg.reply(&ctx.http, "This is a sub function!").await?;

    Ok(())
}
#[command]
// #[aliases("sub-command", "secret")]
#[description("This is `upper`'s sub-command.")]
async fn subscriber_role(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    msg.reply(&ctx.http, "This is a sub function!").await?;

    Ok(())
}

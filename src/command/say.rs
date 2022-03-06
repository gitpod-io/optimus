use super::*;

// Repeats what the user passed as argument but ensures that user and role
// mentions are replaced with a safe textual alternative.
// In this example channel mentions are excluded via the `ContentSafeOptions`.
#[command]
#[required_permissions(ADMINISTRATOR)]
async fn say(_ctx: &Context, _msg: &Message, _args: Args) -> CommandResult {
    // Firstly remove the command msg
    _msg.channel_id.delete_message(&_ctx.http, _msg.id).await?;

    // Use contentsafe options
    let settings = {
        ContentSafeOptions::default()
            .clean_channel(false)
            .clean_role(true)
            .clean_user(false)
    };

    let content = content_safe(&_ctx.cache, &_args.rest(), &settings).await;
    let ref_msg = &_msg.referenced_message;

    if ref_msg.is_some() {
        ref_msg
            .as_ref()
            .map(|x| x.reply_ping(&_ctx.http, &content))
            .unwrap()
            .await?;
    } else {
        _msg.channel_id.say(&_ctx.http, &content).await?;
    }

    Ok(())
}

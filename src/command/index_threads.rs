use serenity::{model::prelude::ChannelId, utils};

use super::*;
use crate::utils::index_threads::index_channel_threads;

// Repeats what the user passed as argument but ensures that user and role
// mentions are replaced with a safe textual alternative.
// In this example channel mentions are excluded via the `ContentSafeOptions`.
#[command]
#[required_permissions(ADMINISTRATOR)]
async fn index_threads(ctx: &Context, _msg: &Message, args: Args) -> CommandResult {
    let mut args = Args::new(args.rest(), &[Delimiter::Single(' ')]);

    let channel_ids = args
        .iter::<String>()
        .filter_map(|i| utils::parse_channel(i.ok()?))
        .map(ChannelId)
        .collect::<Vec<ChannelId>>();

    index_channel_threads(ctx, channel_ids.as_slice()).await?;

    Ok(())
}

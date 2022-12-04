use anyhow::Result;
use serenity::model::application::interaction::Interaction;
use serenity::{client::Context, model::application::interaction::InteractionResponseType};

// Internals
mod close_issue;
mod getting_started;
mod question_thread_suggestions;
mod slash_commands;

pub async fn responder(ctx: &Context, interaction: Interaction) -> Result<()> {
    match interaction {
        Interaction::MessageComponent(mci) => match mci.data.custom_id.as_str() {
            "gitpod_close_issue" => close_issue::responder(&mci, ctx).await?,
            "getting_started_letsgo" => getting_started::responder(&mci, ctx).await,
            _ => question_thread_suggestions::responder(&mci, ctx).await?,
        },
        Interaction::ApplicationCommand(mci) => match mci.data.name.as_str() {
            "close" => close_issue::responder(&mci, ctx).await?,
            "nothing_to_see_here" => {
                slash_commands::nothing_to_see_here::responder(mci, ctx).await?
            }
            _ => {}
        },
        _ => {}
    }

    Ok(())
}

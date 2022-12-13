use anyhow::{Context as _, Result};
use serenity::model::application::interaction::{
    application_command::ApplicationCommandInteraction, MessageFlags,
};
use serenity::{client::Context, model::application::interaction::InteractionResponseType};

pub async fn responder(mci: ApplicationCommandInteraction, ctx: &Context) -> Result<()> {
    let input = mci
        .data
        .options
        .get(0)
        .context("Expected input")?
        .value
        .as_ref()
        .context("Failed ref for input")?;

    mci.create_interaction_response(&ctx.http, |r| {
        r.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|d| {
                d.content("Posted message on this channel")
                    .flags(MessageFlags::EPHEMERAL)
            })
    })
    .await?;

    mci.channel_id
        .send_message(&ctx.http, |m| {
            m.content(
                input
                    .to_string()
                    .trim_start_matches('"')
                    .trim_end_matches('"'),
            )
        })
        .await?;

    Ok(())
}

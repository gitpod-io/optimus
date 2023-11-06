use anyhow::{Context as _, Result};
use serenity::prelude::*;

use serenity::{
    client::Context,
    model::{
        application::{
            component::ButtonStyle,
            interaction::message_component::MessageComponentInteraction,
            interaction::{InteractionResponseType, MessageFlags},
        },
        channel::ReactionType,
        prelude::component::Button,
    },
};

pub async fn responder(mci: &MessageComponentInteraction, ctx: &Context) -> Result<()> {
    if mci.data.custom_id.starts_with("http") {
        let button_label = &mci
            .message
            .components
            .iter()
            .find_map(|a| {
                a.components.iter().find_map(|x| {
                    let button: Button =
                        serde_json::from_value(serde_json::to_value(x).unwrap()).unwrap();
                    if button.custom_id? == mci.data.custom_id {
                        Some(button.label?)
                    } else {
                        None
                    }
                })
            })
            .context("Failed to get button label")?;

        mci.create_interaction_response(&ctx.http, |r| {
            r.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|d| {
                    d.flags(MessageFlags::EPHEMERAL);
                    d.content(format!("{}: {button_label}", &mci.user.mention()))
                        .components(|c| {
                            c.create_action_row(|a| {
                                a.create_button(|b| {
                                    b.label("Open link")
                                        .url(&mci.data.custom_id)
                                        .style(ButtonStyle::Link)
                                })
                            })
                        })
                    // .flags(MessageFlags::EPHEMERAL)
                })
        })
        .await
        .unwrap();

        mci.message
            .react(&ctx.http, ReactionType::Unicode("🔎".to_string()))
            .await
            .unwrap();
    }

    Ok(())
}

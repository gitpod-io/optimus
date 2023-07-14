use anyhow::Result;
use serde_json::json;
use serenity::model::application::component::ActionRowComponent;
use serenity::{client::Context, model::application::interaction::Interaction};

// Internals
mod close_thread;
mod getting_started;
mod question_thread_suggestions;
mod slash_commands;

use serenity::model::application::interaction::modal::ModalSubmitInteraction;

use crate::BOT_CONFIG;
async fn company_name_submitted_response(
    mci: &ModalSubmitInteraction,
    ctx: &Context,
) -> Result<()> {
    mci.create_interaction_response(&ctx.http, |r| {
        r.kind(serenity::model::prelude::InteractionResponseType::UpdateMessage)
            .interaction_response_data(|d| {
                d.content("**[4/4]**: You have personalized the server, congrats!")
                    .components(|c| c)
            })
    })
    .await?;
    if let Some(component) = &mci.data.components.get(0)
    && let Some(input_field) = component.components.get(0)
    && let ActionRowComponent::InputText(it) = input_field 
    && let Some(config) = BOT_CONFIG.get() && let Some(misc) = &config.misc 
    {
        reqwest::Client::new()
            .post(&misc.company_share_endpoint)
            .json(&json!(
            {
                "username": mci.user.name,
                "company": it.value
            }
        )).send().await?;
    }
    Ok(())
}

pub async fn responder(ctx: &Context, interaction: Interaction) -> Result<()> {
    match interaction {
        Interaction::MessageComponent(mci) => match mci.data.custom_id.as_str() {
            "gitpod_close_issue" => close_thread::responder(&mci, ctx).await?,
            "getting_started_letsgo" => getting_started::responder(&mci, ctx).await?,
            _ => question_thread_suggestions::responder(&mci, ctx).await?,
        },
        Interaction::ApplicationCommand(mci) => match mci.data.name.as_str() {
            "close" => close_thread::responder(&mci, ctx).await?,
            "nothing_to_see_here" => {
                slash_commands::nothing_to_see_here::responder(mci, ctx).await?
            }
            "create-pr" => slash_commands::create_pr::responder(&mci, ctx).await?,

            _ => {}
        },
        Interaction::ModalSubmit(mci) => if mci.data.custom_id.as_str() == "company_name_submitted" {
            company_name_submitted_response(&mci, ctx).await?
        }

        _ => {}
    }

    Ok(())
}

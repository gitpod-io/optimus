use serenity::model::{application::interaction::Interaction, id::ChannelId};
use serenity::prelude::*;

use serenity::{
    client::Context,
    model::{
        application::{
            component::ButtonStyle, interaction::message_component::MessageComponentInteraction,
            interaction::InteractionResponseType,
        },
        channel::ReactionType,
        prelude::component::Button,
    },
};

const QUESTIONS_CHANNEL: ChannelId = if cfg!(debug_assertions) {
    ChannelId(1026115789721444384)
} else {
    ChannelId(1026792978854973460)
};

const SELFHOSTED_QUESTIONS_CHANNEL: ChannelId = if cfg!(debug_assertions) {
    ChannelId(1026800568989143051)
} else {
    ChannelId(1026800700002402336)
};

pub async fn responder(mci: &MessageComponentInteraction, ctx: &Context) {
    // If a Question thread suggestion was clicked
    if mci.data.custom_id.starts_with("http") {
        let button_label = &mci
            .message
            .components
            .iter()
            .find_map(|a| {
                a.components.iter().find_map(|x| {
                    let button: Button =
                        serde_json::from_value(serde_json::to_value(x).unwrap()).unwrap();
                    if button.custom_id.unwrap() == mci.data.custom_id {
                        Some(button.label.unwrap())
                    } else {
                        None
                    }
                })
            })
            .unwrap();

        mci.create_interaction_response(&ctx.http, |r| {
            r.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|d| {
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
}

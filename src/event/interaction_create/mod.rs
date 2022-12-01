use super::{QUESTIONS_CHANNEL, SELFHOSTED_QUESTIONS_CHANNEL};
use serenity::model::{application::interaction::Interaction};

use serenity::{client::Context, model::application::interaction::InteractionResponseType};
use serenity::model::application::interaction::MessageFlags;

// Internals
mod close_issue;
mod getting_started;
mod question_thread_suggestions;

pub async fn responder(ctx: Context, interaction: Interaction) {
    let ctx = &ctx.clone();

    match interaction {
        Interaction::MessageComponent(mci) => match mci.data.custom_id.as_str() {
            "gitpod_close_issue" => close_issue::responder(&mci, ctx).await.unwrap(),
            "getting_started_letsgo" => getting_started::responder(&mci, ctx).await,
            _ => question_thread_suggestions::responder(&mci, ctx).await,
        },
        Interaction::ApplicationCommand(mci) => match mci.data.name.as_str() {
            "close" => {
                close_issue::responder(&mci, ctx).await.unwrap();
            }
            "nothing_to_see_here" => {
                let input = mci
                    .data
                    .options
                    .get(0)
                    .expect("Expected input")
                    .value
                    .as_ref()
                    .unwrap();
                mci.create_interaction_response(&ctx.http, |r| {
                    r.kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|d| {
                            d.content("Posted message on this channel")
                                .flags(MessageFlags::EPHEMERAL)
                        })
                })
                .await
                .unwrap();

                mci.channel_id
                    .send_message(&ctx.http, |m| {
                        m.content(
                            input
                                .to_string()
                                .trim_start_matches('"')
                                .trim_end_matches('"'),
                        )
                    })
                    .await
                    .unwrap();
            }
            _ => {}
        },
        _ => {}
    }
}

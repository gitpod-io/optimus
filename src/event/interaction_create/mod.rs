use serenity::model::{application::interaction::Interaction, id::ChannelId};

use serenity::{client::Context, model::application::interaction::InteractionResponseType};

// Internals
mod close_issue;
mod getting_started;
mod question_thread_suggestions;

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

use serenity::model::application::interaction::MessageFlags;

pub async fn responder(ctx: Context, interaction: Interaction) {
    let ctx = &ctx.clone();

    match interaction {
        Interaction::MessageComponent(mci) => match mci.data.custom_id.as_str() {
            "gitpod_close_issue" => close_issue::responder(&mci, ctx).await,
            "getting_started_letsgo" => getting_started::responder(&mci, ctx).await,
            _ => question_thread_suggestions::responder(&mci, ctx).await,
        },
        Interaction::ApplicationCommand(mci) => match mci.data.name.as_str() {
            "close" => {
                let thread_node = mci
                    .channel_id
                    .to_channel(&ctx.http)
                    .await
                    .unwrap()
                    .guild()
                    .unwrap();
                let thread_type = {
                    if [QUESTIONS_CHANNEL, SELFHOSTED_QUESTIONS_CHANNEL]
                        .contains(&thread_node.parent_id.unwrap())
                    {
                        "question"
                    } else {
                        "thread"
                    }
                };
                mci.create_interaction_response(&ctx.http, |r| {
                    r.kind(InteractionResponseType::ChannelMessageWithSource);
                    r.interaction_response_data(|d| {
                        d.content(format!("This {} was closed", thread_type))
                    })
                })
                .await
                .unwrap();
                let thread_name = {
                    if thread_node.name.contains('✅') || thread_type == "thread" {
                        thread_node.name
                    } else {
                        format!("✅ {}", thread_node.name.trim_start_matches("❓ "))
                    }
                };
                mci.channel_id
                    .edit_thread(&ctx.http, |t| t.archived(true).name(thread_name))
                    .await
                    .unwrap();
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

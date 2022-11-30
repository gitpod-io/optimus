
use serenity::model::{application::interaction::Interaction, guild::Member, id::ChannelId};
use serenity::prelude::*;

use serenity::{
    client::Context,
    model::{
        application::{component::ButtonStyle, interaction::InteractionResponseType},
        channel::ReactionType,
    },
};

// Internals
use crate::db::ClientContextExt;

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

use serenity::{
    // http::AttachmentType,
    model::{
        self,
        application::interaction::{message_component::MessageComponentInteraction, MessageFlags},
        guild::Role,
        id::RoleId,
        prelude::component::Button,
        Permissions,
    },
};

pub async fn responder(mci: &MessageComponentInteraction, ctx: &Context) {
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

    let thread_name = {
        if thread_node.name.contains('✅') || thread_type == "thread" {
            thread_node.name
        } else {
            format!("✅ {}", thread_node.name.trim_start_matches("❓ "))
        }
    };
    let action_user_mention = mci.member.as_ref().unwrap().mention();
    let response = format!("This {} was closed by {}", thread_type, action_user_mention);
    mci.channel_id.say(&ctx.http, &response).await.unwrap();
    mci.create_interaction_response(&ctx.http, |r| {
        r.kind(InteractionResponseType::UpdateMessage);
        r.interaction_response_data(|d| d)
    })
    .await
    .unwrap();

    mci.channel_id
        .edit_thread(&ctx.http, |t| t.archived(true).name(thread_name))
        .await
        .unwrap();
}

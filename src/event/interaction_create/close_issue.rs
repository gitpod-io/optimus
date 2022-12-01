use crate::event::{QUESTIONS_CHANNEL, SELFHOSTED_QUESTIONS_CHANNEL};
use anyhow::Result;
use async_trait::async_trait;
use duplicate::duplicate_item;

use serenity::{
    client::Context,
    model::{
        application::interaction::{
            application_command::ApplicationCommandInteraction,
            message_component::MessageComponentInteraction, InteractionResponseType,
        },
        guild::Member,
        id::ChannelId,
    },
    prelude::*,
};

#[async_trait]
pub trait CommonInteractionComponent {
    async fn get_channel_id(&self) -> ChannelId;
    async fn get_member(&self) -> Option<&Member>;
    async fn make_interaction_resp(&self, ctx: &Context) -> Result<()>;
}

#[async_trait]
#[duplicate_item(name; [ApplicationCommandInteraction]; [MessageComponentInteraction])]
impl CommonInteractionComponent for name {
    async fn get_channel_id(&self) -> ChannelId {
        self.channel_id
    }

    async fn get_member(&self) -> Option<&Member> {
        self.member.as_ref()
    }

    async fn make_interaction_resp(&self, ctx: &Context) -> Result<()> {
        self.create_interaction_response(&ctx.http, |r| {
            r.kind(InteractionResponseType::UpdateMessage);
            r.interaction_response_data(|d| d)
        })
        .await?;
        Ok(())
    }
}

pub async fn responder<T>(mci: &T, ctx: &Context) -> Result<()>
where
    T: CommonInteractionComponent,
{
    let channel_id = mci.get_channel_id().await;

    let thread_node = channel_id
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

    {
        use anyhow::Context;
        let action_user_mention = mci
            .get_member()
            .await
            .context("Couldn't get member")?
            .mention();
        let response = format!("This {} was closed by {}", thread_type, action_user_mention);
        channel_id.say(&ctx.http, &response).await?;
    }

    mci.make_interaction_resp(ctx).await?;

    channel_id
        .edit_thread(&ctx.http, |t| t.archived(true).name(thread_name))
        .await?;

    Ok(())
}

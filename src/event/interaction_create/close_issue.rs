use crate::event::{QUESTIONS_CHANNEL, SELFHOSTED_QUESTIONS_CHANNEL};
use anyhow::{Context as _, Result};
use async_trait::async_trait;
use duplicate::duplicate_item;

use serenity::{
    client::Context,
    model::{
        application::interaction::{
            application_command::ApplicationCommandInteraction,
            message_component::MessageComponentInteraction, InteractionResponseType,
            InteractionType,
        },
        guild::Member,
        id::ChannelId,
    },
};

#[async_trait]
pub trait CommonInteractionComponent {
    async fn get_channel_id(&self) -> ChannelId;
    async fn get_member(&self) -> Option<&Member>;
    async fn make_interaction_resp(&self, ctx: &Context, thread_type: &str) -> Result<()>;
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

    async fn make_interaction_resp(&self, ctx: &Context, thread_type: &str) -> Result<()> {

        match self.kind {
            InteractionType::ApplicationCommand => {
                self.create_interaction_response(&ctx.http, |r| {
                    r.kind(InteractionResponseType::ChannelMessageWithSource);
                    r.interaction_response_data(|d| {
                        d.content(format!("This {} was closed", thread_type))
                    })
                })
                .await?;
            }
            InteractionType::MessageComponent => {
                let response = format!(
                    "This {} was closed by {}",
                    thread_type,
                    self.get_member().await.context("Failed to get member")?
                );

                self.channel_id.say(&ctx.http, &response).await?;

                self.create_interaction_response(&ctx.http, |r| {
                    r.kind(InteractionResponseType::UpdateMessage);
                    r.interaction_response_data(|d| d)
                })
                .await?;
            }
            _ => {}
        }

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

    mci.make_interaction_resp(ctx, thread_type).await?;

    channel_id
        .edit_thread(&ctx.http, |t| t.archived(true).name(thread_name))
        .await?;

    Ok(())
}

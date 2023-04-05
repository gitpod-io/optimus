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
        .await?
        .guild()
        .context("Failed to get channel info")?;

    if let Some(config) = crate::BOT_CONFIG.get() && let Some(channels) = &config.discord.channels
    && let Some(primary_questions_channel) = channels.primary_questions_channel_id
    && let Some(secondary_questions_channel) = channels.secondary_questions_channel_id  {

        let thread_type = {
            if [primary_questions_channel, secondary_questions_channel].contains(
                &thread_node
                    .parent_id
                    .context("Failed to get parent_id of thread")?,
            ) {
                "question"
            } else {
                "thread"
            }
        };

        let thread_name = {
            if thread_node.name.contains('✅') || thread_type == "thread" {
                thread_node.name.to_owned()
            } else {
                format!("✅ {}", thread_node.name.trim_start_matches("❓ "))
            }
        };

        let interacted_member = mci.get_member().await.context("Failed to get member")?;

        let mut got_admin = false;
        for role in &interacted_member.roles {
            if role.to_role_cached(&ctx.cache).map_or(false, |r| {
                r.has_permission(serenity::model::Permissions::MANAGE_THREADS)
            }) {
                got_admin = true;
                break;
            }
        }

        if interacted_member.user.id
            == thread_node
                .owner_id
                .context("Failed to get owner_id of thread")?
            || got_admin
        {
            mci.make_interaction_resp(ctx, thread_type).await?;

            channel_id
                .edit_thread(&ctx.http, |t| t.archived(true).name(thread_name))
                .await?;
        }
    }

    Ok(())
}

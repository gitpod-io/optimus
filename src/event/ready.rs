use color_eyre::Report;
use serenity::{
    futures::StreamExt,
    model::{
        application::{command::Command, component::ButtonStyle},
        prelude::{
            command::{CommandOptionType, CommandType},
            GuildInfo, ReactionType,
        },
        Permissions,
    },
};

use super::*;

struct Init;
impl Init {
    async fn set_app_cmds(guild: &GuildInfo, ctx: &Context) -> Result<Vec<Command>, Report> {
        let cmds = GuildId::set_application_commands(&guild.id, &ctx.http, |commands| {
            commands.create_application_command(|command| {
                command.name("close").description("Close a question")
            });

            commands.create_application_command(|command| {
                command
                    .name("create-pr")
                    .description("Pull this thread into GitHub to merge into website")
                    .kind(CommandType::ChatInput)
                    .default_member_permissions(Permissions::ADMINISTRATOR)
                    .create_option(|opt| {
                        opt.kind(CommandOptionType::String)
                            .name("link")
                            .description("Page link to a https://www.gitpod.io/docs/<page>")
                            .required(true)
                    })
                    .create_option(|opt| {
                        opt.kind(CommandOptionType::String)
                            .name("title")
                            .description("Title of the FAQ")
                            .required(false)
                    })
            });

            commands.create_application_command(|c| {
                c.name("nothing_to_see_here")
                    .description("Nope :P")
                    .kind(CommandType::ChatInput)
                    .default_member_permissions(Permissions::ADMINISTRATOR)
                    .create_option(|opt| {
                        opt.kind(CommandOptionType::String)
                            .name("value")
                            .description(";-;")
                            .required(true)
                    })
            });

            commands
        })
        .await?;

        Ok(cmds)
    }

    async fn install_getting_started_message(ctx: &Context) -> Result<()> {
        let placeholder_text = "**Press the button below** 👇 to gain access to the server";

        if let Some(config) = crate::BOT_CONFIG.get() && let Some(channels) = &config.discord.channels
        && let Some(getting_started_channel) = channels.getting_started_channel_id {

            let mut cursor = getting_started_channel.messages_iter(&ctx.http).boxed();
            while let Some(message_result) = cursor.next().await {
                if let Ok(message) = message_result &&message.content == *placeholder_text {
                        return Ok(())
                }
            }

            getting_started_channel
                .send_message(&ctx.http, |m| {
                    m.content(placeholder_text);
                    m.components(|c| {
                        c.create_action_row(|a| {
                            a.create_button(|b| {
                                b.label("Let's go")
                                    .custom_id("getting_started_letsgo")
                                    .style(ButtonStyle::Primary)
                                    .emoji(ReactionType::Unicode("🙌".to_string()))
                            })
                        })
                    });
                    m
                })
                .await?;
        }

        Ok(())
    }
}

// Main entrypoint
pub async fn responder(ctx: &Context, ready: Ready) -> Result<()> {
    println!("{} is connected!", ready.user.name);
    ctx.set_activity(Activity::watching("The pods on Gitpod!"))
        .await;

    let guilds = &ready.user.guilds(&ctx.http).await?;
    for guild in guilds {
        Init::set_app_cmds(guild, ctx).await?;
    }

    Init::install_getting_started_message(ctx).await?;

    Ok(())
}

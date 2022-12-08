use super::*;
use anyhow::Result;
use serenity::model::{
    prelude::command::{CommandOptionType, CommandType},
    Permissions,
};

pub async fn responder(_ctx: &Context, ready: Ready) -> Result<()> {
    println!("{} is connected!", ready.user.name);
    _ctx.set_activity(Activity::watching("The pods on Gitpod!"))
        .await;

    let guilds = &ready.user.guilds(&_ctx.http).await?;

    for guild in guilds {
        // let commands =
        GuildId::set_application_commands(&guild.id, &_ctx.http, |commands| {
            commands.create_application_command(|command| {
                command.name("close").description("Close a question")
            });

            commands.create_application_command(|command| {
                command.name("pull").description("Pull this thread into GitHub to merge into website")
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
    }

    // println!(
    //     "Now I have these application commands: {}",
    //     commands
    //         .into_iter()
    //         .map(|x| format!("{} ", x.name))
    //         .collect::<String>()
    // );

    Ok(())
}

use serenity::model::{
    prelude::command::{CommandOptionType, CommandType},
    Permissions,
};

use super::*;

pub async fn responder(_ctx: &Context, ready: Ready) {
    println!("{} is connected!", ready.user.name);
    _ctx.set_activity(Activity::watching("The pods on Gitpod!"))
        .await;

    let guild_id = GuildId(
        env::var("GUILD_ID")
            .expect("Expected GUILD_ID in environment")
            .parse()
            .expect("GUILD_ID must be an integer"),
    );

    let commands = GuildId::set_application_commands(&guild_id, &_ctx.http, |commands| {
        commands.create_application_command(|command| {
            command.name("close").description("Close a question")
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
    .await
    .unwrap();

    println!(
        "Now I have these application commands: {}",
        commands
            .into_iter()
            .map(|x| format!("{} ", x.name))
            .collect::<String>()
    );

    questions_thread::responder(_ctx).await;
}

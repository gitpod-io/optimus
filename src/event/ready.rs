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

    let _commands = GuildId::set_application_commands(&guild_id, &_ctx.http, |commands| {
        commands.create_application_command(|command| {
            command.name("ask").description("Ask a question")
        });
        commands.create_application_command(|command| {
            command.name("close").description("Close a question")
        })
    })
    .await
    .unwrap();
}

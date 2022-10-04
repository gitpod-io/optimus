use serenity::model::id::ChannelId;

use super::*;

pub struct QuestionChannels {
    pub id: ChannelId,
}

impl Db {
    pub async fn set_watch_channels(
        &self,
        data_name: &str,
        id: ChannelId,
        _ctx: &Context,
    ) -> Result<()> {
        let id = id.0 as i64;

        let q = format!("insert into server_config({}) values({})", data_name, id);
        if sqlx::query(&q).execute(&self.sqlitedb).await.is_ok() {
            // questions_thread::responder(ctx).await;
        }
        Ok(())
    }
    pub async fn get_question_channels(&self) -> Result<Vec<QuestionChannels>> {
        let q = sqlx::query!("select question_channels from server_config")
            .fetch_all(&self.sqlitedb)
            .await?
            .into_iter()
            .map(|x| QuestionChannels {
                id: ChannelId(x.question_channels as u64),
            })
            .collect();

        Ok(q)
    }
}

// A command can have sub-commands, just like in command lines tools.
// Imagine `cargo help` and `cargo help run`.
#[command("config")]
// #[sub_commands(
//     questions_channel,
//     introduction_channel,
//     subscriber_role,
//     getting_started,
//     default_role
// )]
#[required_permissions(ADMINISTRATOR)]
async fn config(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    if _args.is_empty() {
        msg.reply(&ctx.http, format!("{} no argument provided", _args.rest()))
            .await?;
    } else {
        let mut arg = Args::new(_args.rest(), &[Delimiter::Single(' ')]);
        let config = arg.single_quoted::<String>()?;
        let value = arg.single_quoted::<String>()?;
        let db = &ctx.get_db().await;

        match config.as_str() {
            "question_channels"
            | "feedback_channel"
            | "getting_started_channel"
            | "introduction_channel"
            | "showcase_channel" => {
                db.set_watch_channels(config.as_str(), value.parse::<ChannelId>().unwrap(), ctx)
                    .await?
            }
            _ => {
                msg.reply(
                    &ctx.http,
                    format!("Invalid config parameter: {}", _args.rest()),
                )
                .await?;
            }
        }
    }

    Ok(())
}

// // This will only be called if preceded by the `upper`-command.
// #[command]
// // #[aliases("sub-command", "secret")]
// #[description("Set the question channels to watch for")]
// async fn questions_channel(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
//     impl Db {}
//     let db = &ctx.get_db().await;
//     Ok(())
// }
// #[command]
// // #[aliases("sub-command", "secret")]
// #[description("Set the introduction channel to watch for")]
// async fn introduction_channel(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
//     msg.reply(&ctx.http, "This is a sub function!").await?;

//     Ok(())
// }
// #[command]
// // #[aliases("sub-command", "secret")]
// #[description("Set the getting started channel to handle")]
// async fn getting_started(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
//     msg.reply(&ctx.http, "This is a sub function!").await?;

//     Ok(())
// }
// #[command]
// // #[aliases("sub-command", "secret")]
// #[description("Set the subscriber role for users")]
// async fn subscriber_role(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
//     msg.reply(&ctx.http, "This is a sub function!").await?;

//     Ok(())
// }
// #[command]
// #[description("Set the default role for server members")]
// async fn default_role(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
//     Ok(())
// }

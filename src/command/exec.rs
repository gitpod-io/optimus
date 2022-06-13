use super::*;
use regex::Regex;
use serenity::utils::MessageBuilder;

#[command]
#[only_in(guilds)]
#[aliases("sh")]
async fn exec(ctx: &Context, _msg: &Message, mut _args: Args) -> CommandResult {
    let typing = _msg.channel_id.start_typing(&ctx.http)?;
    let args = &_args.rest();
    if let Some(input) = Regex::new(r#"```(?P<lang>[a-z]+)(?s)(?P<code>\n.*)```"#)?.captures(args) {
        let lang = input.name("lang").unwrap().as_str();
        let code = input.name("code").unwrap().as_str();

        let client = piston_rs::Client::new();
        let executor = piston_rs::Executor::new()
            .set_language(lang)
            .set_version("*")
            .add_file(piston_rs::File::default().set_content(code));

        let mut final_msg = String::new();
        match client.execute(&executor).await {
            Ok(response) => {
                if let Some(c) = response.compile {
                    if c.code != 0 {
                        final_msg.push_str(c.output.as_str());
                    }
                }

                if final_msg.is_empty() {
                    final_msg.push_str(response.run.output.as_str());
                }
            }
            Err(e) => {
                final_msg.push_str(format!("Error: Something went wrong: {e}").as_str());
            }
        }
        if final_msg.is_empty() {
            _msg.reply_ping(&ctx.http, "Error: No output received")
                .await?;
        } else {
            _msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.reference_message(_msg).embed(|e| {
                        e.description(format!("```{}\n{}```", lang, final_msg.substring(0, 4070)))
                    })
                })
                .await?;
        }
    } else {
        let final_msg = MessageBuilder::new()
            .push_quote_line("Incorrect syntax, the correct syntax is:\n")
            .push_line("gp exec")
            .push_line("\\`\\`\\`<lang>")
            .push_line("		<code goes here>")
            .push_line("\\`\\`\\`")
            .build();
        _msg.reply_ping(&ctx.http, final_msg).await?;
    }
    typing.stop().unwrap();

    Ok(())
}

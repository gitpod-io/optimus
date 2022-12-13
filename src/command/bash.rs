use super::*;
use urlencoding::encode;

// Repeats what the user passed as argument but ensures that user and role
// mentions are replaced with a safe textual alternative.
// In this example channel mentions are excluded via the `ContentSafeOptions`.
#[command]
#[only_in(guilds)]
#[aliases("sh")]
async fn bash(_ctx: &Context, _msg: &Message, mut _args: Args) -> CommandResult {
    // // Firstly remove the command msg
    // _msg.channel_id
    //     .delete_message(&_ctx.http, _msg.id)
    //     .await
    //     .ok();

    // // Use contentsafe options
    // let settings = {
    //     ContentSafeOptions::default()
    //         .clean_channel(false)
    //         .clean_role(true)
    //         .clean_user(false)
    // };

    // let content = content_safe(&_ctx.cache, &_args.rest(), &settings).await;
    // _msg.channel_id.say(&_ctx.http, &content).await?;

    if _msg.author.id == 465353539363930123 {
        let cmd_args = &_args.rest();
        // println!("{:?}", cmd_prog);

        if !cmd_args.contains("kill") {
            let typing = _ctx
                .http
                .start_typing(u64::try_from(_msg.channel_id).unwrap())
                .unwrap();
            let cmd_output = process::Command::new("bash")
                .arg("-c")
                .arg(cmd_args)
                .output()
                .await
                .unwrap();
            let cmd_stdout = String::from_utf8_lossy(&cmd_output.stdout);
            let cmd_stderr = String::from_utf8_lossy(&cmd_output.stderr);
            let stripped_cmd = &_args.rest().replace('\n', "; ");
            let encoded_url = encode(stripped_cmd);
            // println!("{}", &cmd_output.stderr);
            _msg.channel_id
                .send_message(&_ctx.http, |m| {
                    // m.content("test");
                    // m.tts(true);

                    m.embed(|e| {
                        e.title("Bash command");
                        e.description(format!(
                            "[{}](https://explainshell.com/explain?cmd={})",
                            &stripped_cmd, &encoded_url
                        ));
                        e.field(
                            "Standard output:",
                            format!(
                                "{}{}{}",
                                "```\n",
                                &cmd_stdout.to_string().as_str().substring(0, 1016),
                                "```\n"
                            ),
                            false,
                        );
                        e.field(
                            "Standard error:",
                            format!(
                                "{}{}{}",
                                "```\n",
                                &cmd_stderr.to_string().as_str().substring(0, 1016),
                                "```\n"
                            ),
                            false,
                        );

                        e
                    });

                    m
                })
                .await
                .unwrap();
            typing.stop().unwrap();
        }
    } else {
        _msg.reply(&_ctx.http, "Not available for you")
            .await
            .unwrap();
    }

    Ok(())
}

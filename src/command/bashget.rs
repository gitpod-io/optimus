use super::*;

#[command]
async fn bashget(_ctx: &Context, _msg: &Message, _args: Args) -> CommandResult {
    if path::Path::new(&_args.rest()).exists() {
        _msg.channel_id
            .send_message(&_ctx.http, |m| {
                // m.content("test");
                // m.tts(true);

                m.embed(|e| {
                    e.title("Bash command");
                    e.description(&_args.rest());
                    e.attachment(format!("attachment://{}", &_args.rest()));

                    e
                });

                m
            })
            .await
            .unwrap();
        let paths = vec!["/workspace/dinobot/heh.txt"];

        _msg.channel_id
            .send_files(&_ctx.http, paths, |m| m.content("l"))
            .await
            .unwrap();
    }
    Ok(())
}

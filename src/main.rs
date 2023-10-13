#![feature(let_chains)]

mod command;
mod config;
mod event;
mod init;
mod utils;

use color_eyre::eyre::Result;
use command::*;
use once_cell::sync::OnceCell;
use serenity::{
    framework::standard::{buckets::LimitedFor, StandardFramework},
    http::Http,
    prelude::*,
};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::{atomic::AtomicBool, Arc},
};

static BOT_CONFIG: OnceCell<config::BotConfig> = OnceCell::new();

#[tokio::main]
async fn main() -> Result<()> {
    // Tracing
    init::tracing()?;
    color_eyre::install()?;

    // Read BotConfig.toml
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "BotConfig.toml".to_owned());
    let config = BOT_CONFIG.get_or_try_init(|| config::read(Path::new(&config_path)))?;

    // Init meilisearch
    if let Some(meili) = &config.meilisearch {
        init::meilisearch(meili).await?;
    }

    let http = Http::new(&config.discord.bot_token);
    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => panic!("Could not access the bot id: {:?}", why),
            }
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| {
            c.with_whitespace(true)
                .on_mention(Some(bot_id))
                .prefix("gp")
                // In this case, if "," would be first, a message would never
                // be delimited at ", ", forcing you to trim your arguments if you
                // want to avoid whitespaces at the start of each.
                .delimiters(vec![", ", ","])
                // Sets the bot's owners. These will be used for commands that
                // are owners only.
                .owners(owners)
        })
        // Set a function to be called prior to each command execution. This
        // provides the context of the command, the message that was received,
        // and the full name of the command that will be called.
        //
        // Avoid using this to determine whether a specific command should be
        // executed. Instead, prefer using the `#[check]` macro which
        // gives you this functionality.
        //
        // **Note**: Async closures are unstable, you may use them in your
        // application if you are fine using nightly Rust.
        // If not, we need to provide the function identifiers to the
        // hook-functions (before, after, normal, ...).
        .before(before)
        // Similar to `before`, except will be called directly _after_
        // command execution.
        ////// .after(after)
        // Set a function that's called whenever an attempted command-call's
        // command could not be found.
        ////// .unrecognised_command(unknown_command)
        // Set a function that's called whenever a message is not a command.
        ////// .normal_message(normal_message)
        // Set a function that's called whenever a command's execution didn't complete for one
        // reason or another. For example, when a user has exceeded a rate-limit or a command
        // can only be performed by the bot owner.
        // .on_dispatch_error(dispatch_error)
        // Can't be used more than once per 5 seconds:
        .bucket("emoji", |b| b.delay(5))
        .await
        // Can't be used more than 2 times per 30 seconds, with a 5 second delay applying per channel.
        // Optionally `await_ratelimits` will delay until the command can be executed instead of
        // cancelling the command invocation.
        .bucket("complicated", |b| {
            b.limit(2)
                .time_span(30)
                .delay(5)
                // The target each bucket will apply to.
                .limit_for(LimitedFor::Channel)
                // The maximum amount of command invocations that can be delayed per target.
                // Setting this to 0 (default) will never await/delay commands and cancel the invocation.
                .await_ratelimits(1)
                // A function to call when a rate limit leads to a delay.
                .delay_action(delay_action)
        })
        .await
        // The `#[group]` macro generates `static` instances of the options set for the group.
        // They're made in the pattern: `#name_GROUP` for the group instance and `#name_GROUP_OPTIONS`.
        // #name is turned all uppercase
        .help(&MY_HELP)
        .group(&GENERAL_GROUP);
    // .group(&NOTE_GROUP);
    ////// .group(&EMOJI_GROUP)
    ////// .group(&MATH_GROUP)
    // .group(&OWNER_GROUP)

    let mut client = Client::builder(&config.discord.bot_token, GatewayIntents::default())
        // .application_id(config.discord.application_id)
        .event_handler(event::Listener {
            is_loop_running: AtomicBool::new(false),
        })
        .intents(GatewayIntents::all())
        .framework(framework)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<CommandCounter>(HashMap::default());
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }

    Ok(())
}

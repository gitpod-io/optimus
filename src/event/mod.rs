mod guild_create;
mod guild_member_addition;
mod guild_member_removal;
mod interaction_create;
mod message;
mod message_delete;
mod message_update;
mod questions_thread;
mod reaction_add;
mod ready;
mod thread_update;

use crate::utils::{db::*, misc::vowel_gen, substr};

use regex::Regex;
use serde_json::json;

use serenity::model::channel::GuildChannel;
use serenity::model::interactions::message_component::{ActionRowComponent, InputTextStyle};
use serenity::model::interactions::Interaction;
use serenity::{
    async_trait,
    model::{
        channel::{Message, Reaction},
        event::MessageUpdateEvent,
        gateway::{Activity, Ready},
        guild::{Guild, Member},
        id::{ChannelId, GuildId, MessageId},
        prelude::User,
    },
    prelude::*,
    utils::{content_safe, ContentSafeOptions},
};
use std::convert::TryFrom;

use std::{
    env, path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use anyhow::Result;
use thorne::english_gen;
use tokio::{fs, process};

// questions_thread

use serenity::{
    client::{Context, EventHandler},
    model::{
        channel::ReactionType,
        interactions::{message_component::ButtonStyle, InteractionResponseType},
    },
};

// static QUESTIONS_PLACEHOLDER_TEXT: &str = ">
// > Ask or discuss about anything related with Gitpod
// > ‎";

pub struct Listener {
    pub is_loop_running: AtomicBool,
}

#[async_trait]
impl EventHandler for Listener {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    ///
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.

    async fn message(&self, _ctx: Context, _msg: Message) {
        message::responder(_ctx, _msg).await.unwrap();
    }

    async fn message_delete(
        &self,
        _ctx: Context,
        _channel_id: ChannelId,
        _deleted_message_id: MessageId,
        _guild_id: Option<GuildId>,
    ) {
        message_delete::responder(_ctx, _channel_id, _deleted_message_id, _guild_id).await;
    }

    async fn message_update(
        &self,
        _ctx: Context,
        _old_if_available: Option<Message>,
        _new: Option<Message>,
        _event: MessageUpdateEvent,
    ) {
        message_update::responder(_ctx, _old_if_available, _new, _event).await;
    }

    async fn thread_create(&self, _ctx: Context, _thread: GuildChannel) {
        _thread.id.join_thread(&_ctx.http).await.unwrap();
    }
    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _ctx: Context, ready: Ready) {
        ready::responder(&_ctx, ready).await;
        questions_thread::responder(&_ctx).await;
    }

    async fn guild_member_addition(&self, _ctx: Context, _guild_id: GuildId, _new_member: Member) {
        guild_member_addition::responder(_ctx, _guild_id, _new_member).await;
    }

    async fn guild_member_removal(
        &self,
        _ctx: Context,
        _guild_id: GuildId,
        _user: User,
        _member_data_if_available: Option<Member>,
    ) {
        guild_member_removal::responder(_ctx, _guild_id, _user, _member_data_if_available).await;
    }

    async fn reaction_add(&self, _ctx: Context, _added_reaction: Reaction) {
        reaction_add::responder(_ctx, _added_reaction).await;
    }

    // We use the cache_ready event just in case some cache operation is required in whatever use
    // case you have for this.
    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        println!("Cache built successfully!");

        // it's safe to clone Context, but Arc is cheaper for this use case.
        // Untested claim, just theoretically. :P
        let ctx = Arc::new(ctx);

        // We need to check that the loop is not already running when this event triggers,
        // as this event triggers every time the bot enters or leaves a guild, along every time the
        // ready shard event triggers.
        //
        // An AtomicBool is used because it doesn't require a mutable reference to be changed, as
        // we don't have one due to self being an immutable reference.
        if !self.is_loop_running.load(Ordering::Relaxed) {
            // We have to clone the Arc, as it gets moved into the new thread.
            let ctx1 = Arc::clone(&ctx);
            // tokio::spawn creates a new green thread that can run in parallel with the rest of
            // the application.
            tokio::spawn(async move {
                loop {
                    // We clone Context again here, because Arc is owned, so it moves to the
                    // new function.
                    // log_system_load(Arc::clone(&ctx1)).await;
                    let dbnode_userid = Database::from("userid".to_string()).await;
                    let guilds = &ctx.cache.guilds().await;

                    for guild in guilds.iter() {
                        let members = &ctx1.cache.guild(guild).await.unwrap().members;

                        for (_user_id, _member) in members {
                            // tokio::time::sleep(Duration::from_secs(2)).await;
                            dbnode_userid
                                .save_user_info(_user_id, _member.user.tag())
                                .await;
                        }
                    }

                    // Workaround process uptime limit on free google server
                    tokio::time::sleep(Duration::from_secs(3 * (24 * (60 * 60)))).await;
                    std::process::Command::new(env::current_exe().unwrap())
                        .spawn()
                        .unwrap();
                    std::process::exit(0);
                }
            });

            // Now that the loop is running, we set the bool to true
            self.is_loop_running.swap(true, Ordering::Relaxed);
        }
    }

    async fn thread_update(&self, _ctx: Context, _thread: GuildChannel) {
        thread_update::responder(_ctx, _thread).await;
    }

    async fn guild_create(&self, _ctx: Context, _guild: Guild, _is_new: bool) {
        guild_create::responder(_ctx, _guild, _is_new).await;
    }
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        interaction_create::responder(ctx, interaction).await;
    }
}

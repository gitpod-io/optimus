mod guild_member_addition;
mod guild_member_removal;
mod message;
mod message_delete;
mod reaction_add;
// mod message_update;

use crate::command::note::*;
use crate::utils::db::*;
use glob::*;
use regex::Regex;
use serde_json::json;
use serenity::{
    async_trait,
    model::{
        channel::{Message, Reaction},
        gateway::Ready,
        guild::Member,
        id::{ChannelId, GuildId, MessageId},
        prelude::{User},
    },
    prelude::*,
    utils::{content_safe, ContentSafeOptions},
};
use std::convert::TryFrom;
use std::*;

pub struct Listener;

#[async_trait]
impl EventHandler for Listener {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    ///
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.

    async fn message(&self, _ctx: Context, _msg: Message) {
        message::responder(_ctx, _msg).await;
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

    // async fn message_update(
    //     &self,
    //     _ctx: Context,
    //     _old_if_available: Option<Message>,
    //     _new: Option<Message>,
    //     _event: MessageUpdateEvent,
    // ) {
    //     message_update::responder(_ctx, _old_if_available, _new, _event).await;
    // }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
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
}

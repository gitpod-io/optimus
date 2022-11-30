use super::*;
use crate::db::ClientContextExt;
use interactions::*;

use serenity::{
    futures::StreamExt,
    // http::AttachmentType,
    model::{
        self,
        application::interaction::{message_component::MessageComponentInteraction, MessageFlags},
        guild::Role,
        id::RoleId,
        prelude::component::Button,
        Permissions,
    },
    utils::MessageBuilder,
};

#[derive(Clone, Copy)]
struct SelectMenuSpec<'a> {
    value: &'a str,
    label: &'a str,
    display_emoji: &'a str,
    description: &'a str,
}

async fn safe_text(_ctx: &Context, _input: &String) -> String {
    content_safe(
        &_ctx.cache,
        _input,
        &ContentSafeOptions::default()
            .clean_channel(false)
            .clean_role(true)
            .clean_user(false),
        &[],
    )
}

async fn get_role(
    mci: &model::application::interaction::message_component::MessageComponentInteraction,
    ctx: &Context,
    name: &str,
) -> Role {
    let role = {
        if let Some(result) = mci
            .guild_id
            .unwrap()
            .to_guild_cached(&ctx.cache)
            .unwrap()
            .role_by_name(name)
        {
            result.clone()
        } else {
            let r = mci
                .guild_id
                .unwrap()
                .create_role(&ctx.http, |r| {
                    r.name(&name);
                    r.mentionable(false);
                    r.hoist(false);
                    r
                })
                .await
                .unwrap();
            r.clone()
        }
    };
    if role.name != "Member" && role.name != "Gitpodders" && !role.permissions.is_empty() {
        role.edit(&ctx.http, |r| r.permissions(Permissions::empty()))
            .await
            .unwrap();
    }

    role
}

async fn close_issue(mci: &MessageComponentInteraction, ctx: &Context) {
    let thread_node = mci
        .channel_id
        .to_channel(&ctx.http)
        .await
        .unwrap()
        .guild()
        .unwrap();

    let thread_type = {
        if [QUESTIONS_CHANNEL, SELFHOSTED_QUESTIONS_CHANNEL]
            .contains(&thread_node.parent_id.unwrap())
        {
            "question"
        } else {
            "thread"
        }
    };

    let thread_name = {
        if thread_node.name.contains('✅') || thread_type == "thread" {
            thread_node.name
        } else {
            format!("✅ {}", thread_node.name.trim_start_matches("❓ "))
        }
    };
    let action_user_mention = mci.member.as_ref().unwrap().mention();
    let response = format!("This {} was closed by {}", thread_type, action_user_mention);
    mci.channel_id.say(&ctx.http, &response).await.unwrap();
    mci.create_interaction_response(&ctx.http, |r| {
        r.kind(InteractionResponseType::UpdateMessage);
        r.interaction_response_data(|d| d)
    })
    .await
    .unwrap();

    mci.channel_id
        .edit_thread(&ctx.http, |t| t.archived(true).name(thread_name))
        .await
        .unwrap();
}

async fn assign_roles(
    mci: &MessageComponentInteraction,
    ctx: &Context,
    role_choices: Vec<String>,
    member: &mut Member,
    temp_role: &Role,
    member_role: &Role,
) {
    if role_choices.len() > 1 || !role_choices.iter().any(|x| x == "none") {
        // Is bigger than a single choice or doesnt contain none

        let mut role_ids: Vec<RoleId> = Vec::new();
        for role_name in role_choices {
            if role_name == "none" {
                continue;
            }
            let role = get_role(mci, ctx, role_name.as_str()).await;
            role_ids.push(role.id);
        }
        member.add_roles(&ctx.http, &role_ids).await.unwrap();
        let db = &ctx.get_db().await;
        db.set_user_roles(mci.user.id, role_ids).await.unwrap();
    }

    // Remove the temp role from user
    if member.roles.iter().any(|x| x == &temp_role.id) {
        member.remove_role(&ctx.http, temp_role.id).await.unwrap();
    }
    // Add member role if missing
    if !member.roles.iter().any(|x| x == &member_role.id) {
        member.add_role(&ctx.http, member_role.id).await.unwrap();
    }
}


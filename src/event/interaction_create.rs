use super::*;
use serenity::{
    http::AttachmentType,
    model::{channel::Embed, interactions::message_component::MessageComponentInteraction},
    utils::MessageBuilder,
};

const NO_RESPONSE_TEXT: &str = "_No response_";
const SELF_HOSTED_TEXT: &str = "self-hosted-questions";
const SELF_HOSTED_KUBECTL_COMMAND_PLACEHOLDER: &str = "# Run: kubectl get pods -n <namespace>";

async fn safe_text(_ctx: &Context, _input: &String) -> String {
    content_safe(
        &_ctx.cache,
        _input,
        &ContentSafeOptions::default()
            .clean_channel(false)
            .clean_role(true)
            .clean_user(false),
    )
    .await
}

async fn close_issue(mci: &MessageComponentInteraction, ctx: &Context) {
    // let first_msg = mci
    //     .channel_id
    //     .messages(&ctx.http, |f| f.limit(5))
    //     .await
    //     .unwrap();
    // // dbg!(&first_msg);
    // mci.channel_id
    //     .create_reaction(
    //         &ctx.http,
    //         &first_msg.first().unwrap().id,
    //         ReactionType::Unicode("✅".to_string()),
    //     )
    //     .await
    //     .unwrap();

    let _thread = mci.channel_id.edit_thread(&ctx.http, |t| t).await.unwrap();

    let thread_type = {
        if _thread.name.contains("✅") || _thread.name.contains("❓") {
            "question"
        } else {
            "thread"
        }
    };

    let thread_name = {
        if _thread.name.contains("✅") || thread_type == "thread" {
            _thread.name
        } else {
            format!("✅ {}", _thread.name.trim_start_matches("❓ "))
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

async fn show_issue_form(mci: &MessageComponentInteraction, ctx: &Context) {
    let channel_name = mci.channel_id.name(&ctx.cache).await.unwrap();
    mci.create_interaction_response(&ctx, |r| {
        r.kind(InteractionResponseType::Modal);
        r.interaction_response_data(|d| {
            d.custom_id("gitpod_help_button_press");
            d.title("Template");
            d.components(|c| {
                c.create_action_row(|ar| {
                    ar.create_input_text(|it| {
                        it.style(InputTextStyle::Short)
                            .custom_id("input_title")
                            .required(true)
                            .label("Title")
                            .max_length(100)
                    })
                });
                c.create_action_row(|ar| {
                    ar.create_input_text(|it| {
                        it.style(InputTextStyle::Paragraph)
                            .custom_id("input_description")
                            .label("Description")
                            .required(true)
                            .max_length(4000)
                    })
                });
                c.create_action_row(|ar| {
                    ar.create_input_text(|it| {
                        if channel_name != SELF_HOSTED_TEXT {
                            it.style(InputTextStyle::Short)
                                .custom_id("input_workspace")
                                .label("Workspace affected")
                                .required(false)
                                .max_length(100)
                        } else {
                            it.style(InputTextStyle::Paragraph)
                                .custom_id("input_config_yaml")
                                .label("Your config.yaml contents")
                                .required(false)
                                .max_length(1000)
                        }
                    })
                });
                c.create_action_row(|ar| {
                    ar.create_input_text(|it| {
                        if channel_name != SELF_HOSTED_TEXT {
                            it.style(InputTextStyle::Short)
                                .custom_id("input_example_repo")
                                .label("Example repo")
                                .required(false)
                                .max_length(100)
                        } else {
                            it.style(InputTextStyle::Paragraph)
                                .custom_id("input_kubectl_result")
                                .label("Result of `kubectl get pods -n <namespace>`")
                                .required(false)
                                .max_length(1000)
                                .value(SELF_HOSTED_KUBECTL_COMMAND_PLACEHOLDER)
                        }
                    })
                })
            })
        })
    })
    .await
    .unwrap();
}

pub async fn responder(ctx: Context, interaction: Interaction) {
    let ctx = &ctx.clone();

    match interaction {
        Interaction::MessageComponent(mci) => {
            if mci.data.custom_id == "gitpod_create_issue" {
                show_issue_form(&mci, &ctx).await;
            } else if mci.data.custom_id == "gitpod_close_issue" {
                close_issue(&mci, &ctx).await;
            }
        }
        Interaction::ApplicationCommand(mci) => {
            if mci.data.name == "close" {
                let _thread = mci.channel_id.edit_thread(&ctx.http, |t| t).await.unwrap();
                let thread_type = {
                    if _thread.name.contains("✅") || _thread.name.contains("❓") {
                        "question"
                    } else {
                        "thread"
                    }
                };
                mci.create_interaction_response(&ctx.http, |r| {
                    r.kind(InteractionResponseType::ChannelMessageWithSource);
                    r.interaction_response_data(|d| {
                        d.content(format!("This {} was closed", thread_type))
                    })
                })
                .await
                .unwrap();
                // let thread_id = u64::try_from(mci.channel_id).unwrap();
                // ctx.http
                //     .create_reaction(
                //         QUESTIONS_CHANNEL_ID,
                //         thread_id,
                //         &ReactionType::Unicode("✅".to_string()),
                //     )
                //     .await
                //     .unwrap();
                let thread_node = mci.channel_id.edit_thread(&ctx.http, |t| t).await.unwrap();
                let thread_name = {
                    if thread_node.name.contains("✅") || thread_type == "thread" {
                        thread_node.name
                    } else {
                        format!("✅ {}", thread_node.name.trim_start_matches("❓ "))
                    }
                };
                mci.channel_id
                    .edit_thread(&ctx.http, |t| t.archived(true).name(thread_name))
                    .await
                    .unwrap();
            }
        }
        Interaction::ModalSubmit(mci) => {
            let typing = mci.channel_id.start_typing(&ctx.http).unwrap();
            // dbg!(&mci);
            let title = match mci
                .data
                .components
                .get(0)
                .unwrap()
                .components
                .get(0)
                .unwrap()
            {
                ActionRowComponent::InputText(it) => it,
                _ => return,
            };
            let description = match mci
                .data
                .components
                .get(1)
                .unwrap()
                .components
                .get(0)
                .unwrap()
            {
                ActionRowComponent::InputText(it) => it,
                _ => return,
            };
            let optional_one = match mci
                .data
                .components
                .get(2)
                .unwrap()
                .components
                .get(0)
                .unwrap()
            {
                ActionRowComponent::InputText(it) => {
                    let it = it.clone();
                    if it.value.is_empty() {
                        InputText {
                            custom_id: it.custom_id,
                            kind: it.kind,
                            value: String::from(NO_RESPONSE_TEXT),
                        }
                    } else {
                        it
                    }
                }
                _ => return,
            };
            let optional_two = match mci
                .data
                .components
                .get(3)
                .unwrap()
                .components
                .get(0)
                .unwrap()
            {
                ActionRowComponent::InputText(it) => {
                    let it = it.clone();
                    if it.value.is_empty() {
                        InputText {
                            custom_id: it.custom_id,
                            kind: it.kind,
                            value: String::from(NO_RESPONSE_TEXT),
                        }
                    } else {
                        it
                    }
                }
                _ => return,
            };

            mci.create_interaction_response(ctx, |r| {
                if mci.data.custom_id == "gitpod_help_button_press" {
                    r.kind(InteractionResponseType::ChannelMessageWithSource);
                    r.interaction_response_data(|d| d)
                } else {
                    r.kind(InteractionResponseType::UpdateMessage);
                    r.interaction_response_data(|d| d)
                }
            })
            .await
            .ok();

            let user_name = &mci.user.name;
            let channel_name = &mci.channel_id.name(&ctx.cache).await.unwrap();
            let self_avatar = &ctx.cache.current_user().await.face();
            let self_name = &ctx.cache.current_user().await.name;
            let webhook_get = mci.channel_id.webhooks(&ctx).await.unwrap();
            for hook in webhook_get {
                if hook.name == Some(user_name.clone()) {
                    hook.delete(&ctx).await.unwrap();
                }
            }
            let webhook = mci
                .channel_id
                .create_webhook_with_avatar(
                    &ctx,
                    &user_name,
                    AttachmentType::Image(&mci.user.face().replace(".webp", ".png")),
                )
                .await
                .unwrap();

            let optional_one_safe = safe_text(&ctx, &optional_one.value).await;
            let mut optional_two_safe = safe_text(&ctx, &optional_two.value).await;
            let prepare_embed = Embed::fake(|e| {
                // e.thumbnail(&mci.user.face());
                // e.field("Author", &user_name, false);
                e.field("Title", &title.value, false);
                if channel_name != SELF_HOSTED_TEXT {
                    if optional_one.value != NO_RESPONSE_TEXT {
                        e.field("Workspace affected", &optional_one.value, false);
                    }
                    if optional_two.value != NO_RESPONSE_TEXT {
                        e.field("Example Repository", &optional_two.value, false);
                    }
                } else {
                    let placeholder_one = if optional_one.value == NO_RESPONSE_TEXT {
                        NO_RESPONSE_TEXT
                    } else {
                        "Provided"
                    };
                    let placeholder_two = if optional_two.value == NO_RESPONSE_TEXT
                        || optional_two.value == SELF_HOSTED_KUBECTL_COMMAND_PLACEHOLDER
                    {
                        optional_two_safe.clear();
                        optional_two_safe.push_str(NO_RESPONSE_TEXT);
                        NO_RESPONSE_TEXT
                    } else {
                        "Provided"
                    };
                    if placeholder_one != NO_RESPONSE_TEXT {
                        e.field("config.yaml contents", placeholder_one, false);
                    }
                    if placeholder_two != NO_RESPONSE_TEXT {
                        e.field("Result of kubectl", placeholder_two, false);
                    }
                }
                /*
                e.footer(|f| {
                    f.icon_url(self_avatar);
                    f.text(&self_name)
                })
                */
                e
            });

            let msg = webhook
                .execute(&ctx, true, |w| w.embeds(vec![prepare_embed]))
                .await
                .unwrap()
                .unwrap();
            webhook.delete(&ctx.http).await.unwrap();
            typing.stop();
            if mci.data.custom_id == "gitpod_help_button_press" {
                if let Some(msg) = mci.message {
                    msg.delete(&ctx.http).await.ok();
                }
            }

            let user_mention = mci.user.mention();

            let thread_auto_archive_dur = {
                if cfg!(debug_assertions) {
                    1440 // 1 day
                } else {
                    4320 // 3 days
                }
            };

            let thread = mci
                .channel_id
                .create_public_thread(&ctx, msg.id, |e| {
                    e.name(format!("❓ {}", &title.value))
                        .auto_archive_duration(thread_auto_archive_dur)
                })
                .await
                .unwrap();

            let desc_safe = safe_text(&ctx, &description.value).await;
            thread
                .send_message(&ctx.http, |m| {
                    if &description.value.chars().count() < &1960 {
                        m.content(
                            MessageBuilder::new()
                                .push_underline_line("**Description**")
                                .push_line(&desc_safe)
                                .push_bold("---------------")
                                .build(),
                        );
                    } else {
                        m.add_embed(|e| e.title("Description").description(desc_safe));
                    }
                    if channel_name == SELF_HOSTED_TEXT {
                        if optional_one_safe != NO_RESPONSE_TEXT {
                            m.add_embed(|e| {
                                e.title("config.yaml contents")
                                    .description(format!("```yaml\n{}\n```", optional_one_safe))
                            });
                        }
                        if optional_two_safe != NO_RESPONSE_TEXT {
                            m.add_embed(|e| {
                                e.title("Result of kubectl").description(format!(
                                    "```javascript\n{}\n```",
                                    optional_two_safe
                                ))
                            });
                        }
                    }
                    m
                })
                .await
                .unwrap();

            thread
                .send_message(&ctx, |m| {
                    m.content( MessageBuilder::new().push_quote(format!("Hey {}! Thank you for raising this — please hang tight as someone from our community may help you out. Meanwhile, feel free to add anymore information in this thread!", user_mention)).build()).components(|c| {
                        c.create_action_row(|ar| {
                            ar.create_button(|button| {
                                button
                                    .style(ButtonStyle::Success)
                                    .label("Close")
                                    .custom_id("gitpod_close_issue")
                                    .emoji(ReactionType::Unicode("✉️".to_string()))
                            })
                        })
                    })
                })
                .await
                .unwrap();

            // thread.last_message_id

            questions_thread::responder(&ctx).await;
            // let thread = ctx.http.get_channel(questions_channel_id).await.unwrap().guild().unwrap().create_public_thread(&ctx.http, message_id, f);
        }
        _ => (),
    }
}

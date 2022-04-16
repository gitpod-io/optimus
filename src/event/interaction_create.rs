use super::*;
use serenity::{
    http::AttachmentType,
    model::{channel::Embed, interactions::message_component::MessageComponentInteraction},
    utils::MessageBuilder,
};
use urlencoding::encode;

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

async fn google_site_search_fetch_links(sites: &[&str], query: &str) -> String {
    let mut links = String::new();
    for site in sites.iter() {
        if let Ok(resp) = reqwest::Client::new()
		.get(format!("https://www.google.com/search?q=site:{} {}", encode(&site), encode(query)))
		.header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.182 Safari/537.36")
		.send()
		.await {
			if let Ok(result) = resp.text().await {
				let mut times = 1;
				for caps in
					Regex::new(format!("\"(?P<url>{}/.*?)\"", &site).as_str())
						.unwrap()
						.captures_iter(&result)
				{
					let url = &caps["url"];
					let hash = {
						if let Some(result) = Regex::new(r"(?P<hash>#.*)").unwrap().captures(url) {
							if let Some(hash) = result.name("hash") {
								Some(hash.as_str())
							} else {
								None
							}
						} else {
							None
						}
					};
					if let Ok(resp) = reqwest::Client::new().get(url).header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.182 Safari/537.36")
					.send()
					.await {
						if let Ok(result) = resp.text().await {
							for caps in Regex::new(r"<title>(?P<title>.*?)</title>").unwrap().captures_iter(&result) {
								let title = &caps["title"];
								let text = if hash.is_none() {
									format!("[{}]({})", title, url)
								} else {
									format!("[{}{}]({})", title, hash.unwrap(), url)
								};
								links.push_str(format!("• __{}__\n\n", text).as_str());
							}
						}
					}
					times += 1;
					if times > 3 {
						break;
					}
				}
			}
		}
    }

    links
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
                ActionRowComponent::InputText(it) => it,
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
                ActionRowComponent::InputText(it) => it,
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
            // let self_avatar = &ctx.cache.current_user().await.face();
            // let self_name = &ctx.cache.current_user().await.name;
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

            let temp_embed = Embed::fake(|e| e.description(&description.value));

            let mut msg = webhook
                .execute(&ctx, true, |w| {
                    w.embeds(vec![temp_embed]).content(&title.value)
                })
                .await
                .unwrap()
                .unwrap();
            msg.suppress_embeds(&ctx.http).await.unwrap();
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
                    if channel_name != SELF_HOSTED_TEXT {
                        if !optional_one.value.is_empty() || !optional_two.value.is_empty() {
                            m.add_embed(|e| {
                                if !optional_one.value.is_empty() {
                                    e.field("Workspace affected", &optional_one.value, false);
                                }
                                if !optional_two.value.is_empty() {
                                    e.field("Example Repository", &optional_two.value, false);
                                }
                                e
                            });
                        }
                    } else if channel_name == SELF_HOSTED_TEXT {
                        if !optional_one.value.is_empty() {
                            m.add_embed(|e| {
                                e.title("config.yaml contents")
                                    .description(format!("```yaml\n{}\n```", &optional_one.value))
                            });
                        }
                        if optional_two.value != SELF_HOSTED_KUBECTL_COMMAND_PLACEHOLDER
                            && !optional_two.value.is_empty()
                        {
                            m.add_embed(|e| {
                                e.title("Result of kubectl").description(format!(
                                    "```javascript\n{}\n```",
                                    &optional_two.value
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
                                    .emoji(ReactionType::Unicode("🔒".to_string()))
                            })
                        })
                    })
                })
                .await
                .unwrap();

            questions_thread::responder(&ctx).await;

            let thread_typing = thread.clone().start_typing(&ctx.http).unwrap();
            let relevant_links = google_site_search_fetch_links(
                &["https://www.gitpod.io/docs", "https://github.com/gitpod-io"],
                &title.value,
            )
            .await;
            if !relevant_links.is_empty() {
                thread
                    .send_message(&ctx.http, |m| {
                        m.content(
                            "I also found some relevant links which might answer your question:",
                        )
                        .embed(|e| e.description(relevant_links))
                    })
                    .await
                    .unwrap();
                thread_typing.stop();
            }
        }
        _ => (),
    }
}

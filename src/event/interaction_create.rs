use std::{collections::HashMap, fmt::format};

use super::*;
use crate::db::{self, ClientContextExt, Db};
use anyhow::Result;
use meilisearch_sdk::{client::Client as MeiliClient, settings::Settings};
use serde::{Deserialize, Serialize};

use serenity::{
    http::AttachmentType,
    model::{channel::Embed, interactions::message_component::MessageComponentInteraction},
    utils::MessageBuilder,
};
use urlencoding::encode;

#[derive(Serialize, Deserialize, Debug)]
struct Thread {
    id: u64,
    guild_id: u64,
    channel_id: u64,
    title: String,
    history: String,
}

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

async fn save_and_fetch_links(
    sites: &[&str],
    thread_id: u64,
    channel_id: u64,
    guild_id: u64,
    title: String,
    description: String,
) -> HashMap<String, String> {
    let mut links: HashMap<String, String> = HashMap::new();
    let client = reqwest::Client::new();
    let mclient = MeiliClient::new("http://localhost:7700", "optimus");
    let msettings = Settings::new()
        .with_searchable_attributes(["title", "description"])
        .with_distinct_attribute("title");
    mclient
        .index("threads")
        .set_settings(&msettings)
        .await
        .unwrap();
    let threads = mclient.index("threads");

    // Fetch matching links
    for site in sites.iter() {
        if let Ok(resp) = client
		.get(format!("https://www.google.com/search?q=site:{} {}", encode(site), encode(title.as_str())))
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
							result.name("hash").map(|hash| hash.as_str())
						} else {
							None
						}
					};
					if let Ok(resp) = client.get(url).header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.182 Safari/537.36")
					.send()
					.await {
						if let Ok(result) = resp.text().await {
							for caps in Regex::new(r"<title>(?P<title>.*?)</title>").unwrap().captures_iter(&result) {
								let title = &caps["title"];
								let text = if hash.is_none() {
									title.to_string()
								} else {
									format!("[{} | {}]", title, hash.unwrap())
								};
								//links.push_str(format!("• __{}__\n\n", text).as_str());
								links.insert(text, url.to_string());
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

    // Fetch matching discord questions
    if let Ok(discord_questions) = threads
        .search()
        .with_query(format!("{} {}", title, description).as_str())
        .with_limit(3)
        .execute::<Thread>()
        .await
    {
        for ids in discord_questions.hits {
            links.insert(
                ids.result.title,
                format!(
                    "https://discord.com/channels/{}/{}/{}",
                    ids.result.guild_id, ids.result.channel_id, ids.result.id
                ),
            );
        }
    }

    // Save the question to search engine
    threads
        .add_documents(
            &[Thread {
                id: thread_id,
                channel_id,
                guild_id,
                title,
                history: description,
            }],
            Some("id"),
        )
        .await
        .ok();
    links
}

async fn close_issue(mci: &MessageComponentInteraction, ctx: &Context) {
    let _thread = mci.channel_id.edit_thread(&ctx.http, |t| t).await.unwrap();
    let thread_type = {
        if _thread.name.contains('✅') || _thread.name.contains('❓') {
            "question"
        } else {
            "thread"
        }
    };

    let thread_name = {
        if _thread.name.contains('✅') || thread_type == "thread" {
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
    let db = &ctx.get_db().await;
    let desc = {
        if let Ok(result) = db
            .get_pending_question_content(&mci.user.id, &mci.channel_id)
            .await
        {
            db.remove_pending_question(&mci.user.id, &mci.channel_id)
                .await
                .ok();
            result
        } else {
            String::from("")
        }
    };

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
                            .max_length(98)
                    })
                });
                c.create_action_row(|ar| {
                    ar.create_input_text(|it| {
                        it.style(InputTextStyle::Paragraph)
                            .custom_id("input_description")
                            .label("Description")
                            .required(true)
                            .max_length(4000)
                            .value(desc)
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
                show_issue_form(&mci, ctx).await;
            } else if mci.data.custom_id == "gitpod_close_issue" {
                close_issue(&mci, ctx).await;
            }
        }
        Interaction::ApplicationCommand(mci) => {
            if mci.data.name == "close" {
                let _thread = mci.channel_id.edit_thread(&ctx.http, |t| t).await.unwrap();
                let thread_type = {
                    if _thread.name.contains('✅') || _thread.name.contains('❓') {
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
                let thread_node = mci.channel_id.edit_thread(&ctx.http, |t| t).await.unwrap();
                let thread_name = {
                    if thread_node.name.contains('✅') || thread_type == "thread" {
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

            let desc_safe = safe_text(ctx, &description.value).await;
            thread
                .send_message(&ctx.http, |m| {
                    if description.value.chars().count() < 1960 {
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

            questions_thread::responder(ctx).await;

            let thread_typing = thread.clone().start_typing(&ctx.http).unwrap();
            let mut relevant_links = save_and_fetch_links(
                &["https://www.gitpod.io/docs", "https://github.com/gitpod-io"],
                *thread.id.as_u64(),
                *mci.channel_id.as_u64(),
                *mci.guild_id.unwrap().as_u64(),
                (*title.value).to_string(),
                (*description.value).to_string(),
            )
            .await;
			if !&relevant_links.is_empty() {
			thread.send_message(&ctx.http, |m| {
				m.content(format!("{} I also found some relevant links which might answer your question, please do check them out below 🙏:", &user_mention));
				
					m.components(|c| {
						loop {
							let mut we_done = true;
							dbg!(&relevant_links);
							c.create_action_row(|a|
								{
									let mut i = 1;
									for (mut title, mut url) in relevant_links.clone() {
										if i > 5 {
											we_done = false;
											break;
										}
										i += 1;
									
										title.truncate(80);
										url.truncate(100);
										relevant_links.remove(&title);
										// relevant_links.remove("lol").unwrap();
									
										a.create_button(|b|b.label(&title).custom_id(&url).style(ButtonStyle::Secondary));
										
									}
										a
									}
								);

								if we_done {
									break;
								}

						}
							c
						});
						m
					}
				).await.unwrap();
			}
            // if !relevant_links.is_empty() {
            //     thread
            //         .send_message(&ctx.http, |m| {
            //             m.content(format!(
            //                 "{} I also found some relevant links which might answer your question, please do check them out below 🙏:",
            //                 &user_mention
            //             ))
            //             .embed(|e| e.description(relevant_links))
            //         })
            //         .await
            //         .unwrap();
            //     thread_typing.stop();
            // }
            // let db = &ctx.get_db().await;
            // db.add_title(i64::from(mci.id), &title.value).await.unwrap();
        }
        _ => (),
    }
}

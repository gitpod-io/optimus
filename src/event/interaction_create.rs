use std::collections::HashMap;

use super::*;
use crate::db::ClientContextExt;
use substr::StringUtils;

use meilisearch_sdk::{client::Client as MeiliClient, settings::Settings};
use serde::{Deserialize, Serialize};

use serenity::{
    futures::StreamExt,
    // http::AttachmentType,
    model::{
        self,
        application::interaction::{message_component::MessageComponentInteraction, MessageFlags},
        channel::{AttachmentType, Embed},
        guild::{Emoji, Role},
        id::RoleId,
        prelude::component::Button,
        Permissions,
    },
    utils::{read_image, MessageBuilder},
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
#[derive(Clone, Copy)]
struct SelectMenuSpec<'a> {
    value: &'a str,
    label: &'a str,
    display_emoji: &'a str,
    description: &'a str,
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
				// [^:~] avoids the google hyperlinks
				for caps in
					Regex::new(format!("\"(?P<url>{}/.[^:~]*?)\"", &site).as_str())
						.unwrap()
						.captures_iter(&result)
				{
					let url = &caps["url"];
					let hash = {
						if let Some(result) = Regex::new(r"(?P<hash>#[^:~].*)").unwrap().captures(url) {
							result.name("hash").map(|hash| hash.as_str())
						} else {
							None
						}
					};
					if let Ok(resp) = client.get(url).header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.182 Safari/537.36")
					.send()
					.await {
						if let Ok(result) = resp.text().await {
							let result = html_escape::decode_html_entities(&result).to_string();
							for caps in Regex::new(r"<title>(?P<title>.*?)</title>").unwrap().captures_iter(&result) {
								let title = &caps["title"];
								let text = if hash.is_none() {
									title.to_string()
								} else {
									format!("{} | {}", title, hash.unwrap())
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
            "".to_string()
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
            match mci.data.custom_id.as_str() {
                "gitpod_create_issue" => show_issue_form(&mci, ctx).await,
                "gitpod_close_issue" => close_issue(&mci, ctx).await,
                "getting_started_letsgo" => {
                    let mut additional_roles: Vec<SelectMenuSpec> = Vec::from([
                        SelectMenuSpec {
                            value: "Newcomer",
                            description: "Get to know the people in the community",
                            label: "Newcomer",
                            display_emoji: "🌱",
                        },
                        SelectMenuSpec {
                            value: "Buidler",
                            description: "Find resources and share your work",
                            label: "Buidler",
                            display_emoji: "🏗️",
                        },
                        SelectMenuSpec {
                            value: "EarlyAdopter",
                            description: "Join the pioneers in the ecosystem",
                            label: "Early Adopter",
                            display_emoji: "🌅",
                        },
                        SelectMenuSpec {
                            value: "Governance",
                            description: "Take part in decision making processes",
                            label: "Governance",
                            display_emoji: "🏛️",
                        },
                        SelectMenuSpec {
                            value: "Research",
                            description: "Deep discussions between researchers",
                            label: "Academia and Research",
                            display_emoji: "🧑‍🔬",
                        },
                        SelectMenuSpec {
                            value: "Speculation",
                            description: "Markets, altcoins and degens",
                            label: "Speculation/Degen Stuff",
                            display_emoji: "🏛️",
                        },
                        SelectMenuSpec {
                            value: "AllCategories",
                            description: "Just like the old times",
                            label: "Unlock everything",
                            display_emoji: "♾️",
                        },
                    ]);

                    let mut poll_entries: Vec<SelectMenuSpec> = Vec::from([
                        SelectMenuSpec {
                            value: "Found: FromFriend",
                            label: "Friend or colleague",
                            description: "A friend or colleague of mine introduced IOTA/Shimmer to me",
                            display_emoji: "🫂",
                        },
                        SelectMenuSpec {
                            value: "Found: FromGoogle",
                            label: "Google",
                            description: "I found IOTA/Shimmer from a Google search",
                            display_emoji: "🔎",
                        },
                        SelectMenuSpec {
                            value: "Found: FromYouTube",
                            label: "YouTube",
                            description: "Saw IOTA/Shimmer on a Youtube Video",
                            display_emoji: "📺",
                        },
                        SelectMenuSpec {
                            value: "Found: FromTwitter",
                            label: "Twitter",
                            description: "Saw people talking about IOTA/Shimmer on a Tweet",
                            display_emoji: "🐦",
                        },
                        SelectMenuSpec {
                            value: "Found: FromMarketCap",
                            label: "MarketCap",
                            description: "Found on CoinMarketCap/CoinGecko",
                            display_emoji: "✨",
                        },
                    ]);

                    //for prog_role in [
                    //    "Bash", "C", "CPP", "CSharp", "Docker", "Go", "Haskell", "Java", "Js",
                    //    "Kotlin", "Lua", "Nim", "Nix", "Node", "Perl", "Php", "Python", "Ruby",
                    //    "Rust",
                    //]
                    //.iter()
                    //{
                    //    additional_roles.push(SelectMenuSpec {
                    //        label: prog_role,
                    //        description: "Discussions",
                    //        display_emoji: "📜",
                    //        value: prog_role,
                    //    });
                    //}
                    let mut role_choices: Vec<String> = Vec::new();
                    let mut join_reason = String::new();

                    mci.create_interaction_response(&ctx.http, |r| {
                    r.kind(InteractionResponseType::ChannelMessageWithSource);
                    r.interaction_response_data(|d| {
                        d.content(
                            "**[1/4]:** Which additional channels would you like to have access to?",
                        );
                        d.components(|c| {
                            c.create_action_row(|a| {
                                a.create_select_menu(|s| {
                                    s.placeholder("Select channels (Optional)");
                                    s.options(|o| {
										for spec in &additional_roles {
											o.create_option(|opt| {
												opt.label(spec.label);
												opt.description(spec.description);
												opt.emoji(ReactionType::Unicode(spec.display_emoji.to_string()));
												opt.value(spec.value)
											});
										}
                                        o.create_option(|opt| {
                                            opt.label("[Skip] I don't want any!")
                                                .description("Nopes, I ain't need more.")
                                                .emoji(ReactionType::Unicode("⏭".to_string()))
                                                .value("none");
                                            opt
                                        });
                                        o
                                    });
                                    s.custom_id("channel_choice").max_values(additional_roles.len().try_into().unwrap())
                                });
                                a
                            });
                            c
                        });
                        d.custom_id("bruh")
                            .flags(MessageFlags::EPHEMERAL)
                    });
                    r
                })
                .await
                .unwrap();

                    let mut interactions = mci
                        .get_interaction_response(&ctx.http)
                        .await
                        .unwrap()
                        .await_component_interactions(&ctx)
                        .timeout(Duration::from_secs(60 * 5))
                        .build();

                    while let Some(interaction) = interactions.next().await {
                        match interaction.data.custom_id.as_str() {
                            "channel_choice" => {
                                interaction.create_interaction_response(&ctx.http, |r| {
									r.kind(InteractionResponseType::UpdateMessage).interaction_response_data(|d|{
										d.content("**[2/4]:** Would you like to get notified for announcements and community events?");
										d.components(|c| {
											c.create_action_row(|a| {
												a.create_button(|b|{
													b.label("Yes!").custom_id("subscribed").style(ButtonStyle::Success)
												});
												a.create_button(|b|{
													b.label("No, thank you!").custom_id("not_subscribed").style(ButtonStyle::Danger)
												});
												a
											})
										});
										d
									})
								}).await.unwrap();

                                // Save the choices of last interaction
                                interaction
                                    .data
                                    .values
                                    .iter()
                                    .for_each(|x| role_choices.push(x.to_string()));
                            }
                            "subscribed" | "not_subscribed" => {
                                interaction.create_interaction_response(&ctx.http, |r| {
									r.kind(InteractionResponseType::UpdateMessage).interaction_response_data(|d| {
										d.content("**[3/4]:** Why did you join our community?\nI will point you to the correct channels with this info.").components(|c| {
											c.create_action_row(|a| {
												a.create_button(|b|{
													b.label("To hangout with others");
													b.style(ButtonStyle::Secondary);
													b.emoji(ReactionType::Unicode("🏄".to_string()));
													b.custom_id("hangout")
												});
												a.create_button(|b|{
													b.label("To get help with IOTA/Shimmer");
													b.style(ButtonStyle::Secondary);
													b.emoji(ReactionType::Unicode("✌️".to_string()));
													b.custom_id("gitpodio_help")
												});
												a.create_button(|b|{
													b.label("To develop on IOTA/Shimmer");
													b.style(ButtonStyle::Secondary);
													b.emoji(ReactionType::Unicode("🏡".to_string()));
													b.custom_id("selfhosted_help")
												});
												a
											})
										})
									})
								}).await.unwrap();

                                // Save the choices of last interaction
                                let subscribed_role = SelectMenuSpec {
                                    label: "Events",
                                    description: "Subscribed to event pings",
                                    display_emoji: "",
                                    value: "Events",
                                };
                                if interaction.data.custom_id == "subscribed" {
                                    role_choices.push(subscribed_role.value.to_string());
                                }
                                additional_roles.push(subscribed_role);
                            }
                            "hangout" | "gitpodio_help" | "selfhosted_help" => {
                                interaction.create_interaction_response(&ctx.http, |r| {
									r.kind(InteractionResponseType::UpdateMessage).interaction_response_data(|d| {
										d.content("**[3/4]**: You have personalized the server, congrats!").components(|c|c)
									})
								}).await.unwrap();

                                // Save join reason
                                join_reason.push_str(interaction.data.custom_id.as_str());

                                let mut member = mci.member.clone().unwrap();
                                let member_role = get_role(&mci, ctx, "Member").await;
                                let never_introduced = {
                                    let mut status = true;
                                    if let Some(roles) = member.roles(&ctx.cache) {
                                        let gitpodder_role =
                                            get_role(&mci, ctx, "Gitpodders").await;
                                        status = !roles
                                            .into_iter()
                                            .any(|x| x == member_role || x == gitpodder_role);
                                    }
                                    if status {
                                        let mut count = 0;
                                        if let Ok(intro_msgs) = &ctx
                                            .http
                                            .get_messages(*INTRODUCTION_CHANNEL.as_u64(), "")
                                            .await
                                        {
                                            intro_msgs.iter().for_each(|x| {
                                                if x.author == interaction.user {
                                                    count += 1;
                                                }
                                            });
                                        }

                                        status = count < 1;
                                    }
                                    status
                                };

                                let followup = interaction
                                    .create_followup_message(&ctx.http, |d| {
                                        d.content("**[4/4]:** How did you find Gitpod?");
                                        d.components(|c| {
                                            c.create_action_row(|a| {
                                                a.create_select_menu(|s| {
                                                    s.placeholder(
                                                        "[Poll]: Select sources (Optional)",
                                                    );
                                                    s.options(|o| {
                                                        for spec in &poll_entries {
                                                            o.create_option(|opt| {
                                                                opt.label(spec.label);
                                                                opt.description(spec.description);
                                                                opt.emoji(ReactionType::Unicode(
                                                                    spec.display_emoji.to_string(),
                                                                ));
                                                                opt.value(spec.value);
                                                                opt
                                                            });
                                                        }
                                                        o.create_option(|opt| {
                                                            opt.label("[Skip] Prefer not to share")
                                                                .value("none")
                                                                .emoji(ReactionType::Unicode(
                                                                    "⏭".to_string(),
                                                                ));
                                                            opt
                                                        });
                                                        o
                                                    });
                                                    s.custom_id("found_gitpod_from").max_values(5)
                                                });
                                                a
                                            });
                                            c
                                        });
                                        d.flags(MessageFlags::EPHEMERAL)
                                    })
                                    .await
                                    .unwrap();

                                let temp_role = get_role(&mci, ctx, "Member").await;
                                let followup_results = match followup
                                    .await_component_interaction(&ctx)
                                    .timeout(Duration::from_secs(60 * 5))
                                    .await
                                {
                                    Some(ci) => {
                                        member.add_role(&ctx.http, temp_role.id).await.unwrap();
                                        let final_msg = {
                                            if never_introduced {
                                                MessageBuilder::new()
												.push_line(format!(
													"Thank you {}! To unlock the server, drop by {} :wave:",
													interaction.user.mention(),
													INTRODUCTION_CHANNEL.mention()
												))
												.push_line("\nWe’d love to get to know you better and hear about:")
                                                .push_quote_line("🌈 your favourite IOTA/Shimmer feature")
												.push_quote_line("🔧 what you’re working on!").build()
                                            } else {
                                                "Awesome, your server profile will be updated now!"
                                                    .to_owned()
                                            }
                                        };
                                        ci.create_interaction_response(&ctx.http, |r| {
                                            r.kind(InteractionResponseType::UpdateMessage)
                                                .interaction_response_data(|d| {
                                                    d.content(final_msg).components(|c| c)
                                                })
                                        })
                                        .await
                                        .unwrap();
                                        ci
                                    }
                                    None => return,
                                };

                                // if let Some(interaction) = interaction
                                //     .get_interaction_response(&ctx.http)
                                //     .await
                                //     .unwrap()
                                //     .await_component_interaction(&ctx)
                                //     .timeout(Duration::from_secs(60 * 5))
                                //     .await
                                // {

                                if never_introduced {
                                    // Wait for the submittion on INTRODUCTION_CHANNEL
                                    if let Some(msg) = mci
                                        .user
                                        .await_reply(&ctx)
                                        .timeout(Duration::from_secs(60 * 30))
                                        .await
                                    {
                                        // Watch intro channel
                                        if msg.channel_id == INTRODUCTION_CHANNEL {
                                            // let mut count = 0;
                                            // intro_msgs.iter().for_each(|x| {
                                            // 	if x.author == msg.author {
                                            // 		count += 1;
                                            // 	}
                                            // });

                                            // if count <= 1 {
                                            let thread = msg
                                                .channel_id
                                                .create_public_thread(&ctx.http, &msg.id, |t| {
                                                    t.auto_archive_duration(1440).name(format!(
                                                        "Welcome {}!",
                                                        msg.author.name
                                                    ))
                                                })
                                                .await
                                                .unwrap();

                                            if words_count::count(&msg.content).words > 5 {
                                                msg.react(
                                                    &ctx.http,
                                                    ReactionType::Unicode("🔥".to_string()),
                                                )
                                                .await
                                                .unwrap();
                                            }
                                            msg.react(
                                                &ctx.http,
                                                ReactionType::Unicode("👋".to_string()),
                                            )
                                            .await
                                            .unwrap();

                                            let general_channel = if cfg!(debug_assertions) {
                                                ChannelId(947769443516284943)
                                            } else {
                                                ChannelId(970953101894889523)
                                            };
                                            let offtopic_channel = if cfg!(debug_assertions) {
                                                ChannelId(947769443793141769)
                                            } else {
                                                ChannelId(970953101894889529)
                                            };
                                            let db = &ctx.get_db().await;
                                            let questions_channel =
                                                db.get_question_channels().await.unwrap();
                                            let questions_channel =
                                                questions_channel.into_iter().next().unwrap().id;

                                            let selfhosted_questions_channel =
                                                if cfg!(debug_assertions) {
                                                    ChannelId(947769443793141761)
                                                } else {
                                                    ChannelId(879915120510267412)
                                                };

                                            let mut prepared_msg = MessageBuilder::new();
                                            prepared_msg.push_line(format!(
                                                "Welcome to the IOTA/Shimmer community {} 🙌\n",
                                                &msg.author.mention()
                                            ));
                                            match join_reason.as_str() {
                                                "gitpodio_help" => {
                                                    prepared_msg.push_line(
														format!("**You mentioned that** you need help with Gitpod.io, please ask in {}\n",
																	&questions_channel.mention())
													);
                                                }
                                                "selfhosted_help" => {
                                                    let selfhosted_role =
                                                        get_role(&mci, ctx, "SelfHosted").await;
                                                    member
                                                        .add_role(&ctx.http, selfhosted_role.id)
                                                        .await
                                                        .unwrap();
                                                    prepared_msg.push_line(
														format!("**You mentioned that** you need help with selfhosted, please ask in {}\n",
																	&selfhosted_questions_channel.mention())
													);
                                                }
                                                _ => {}
                                            }
                                            prepared_msg.push_bold_line("Here are some channels that you should check out:")
											.push_quote_line(format!("• {} - for anything IOTA/Shimmer related", &general_channel.mention()))
											.push_quote_line(format!("• {} - for any random discussions ☕️", &offtopic_channel.mention()))
											.push_quote_line(format!("• {} - have a question or need help? This is the place to ask! ❓\n", &questions_channel.mention()))
											.push_line("…And there’s more! Take your time to explore :)\n")
											.push_bold_line("Feel free to check out the following pages to learn more about IOTA/Shimmer:")
											.push_quote_line("• https://www.iota.org")
                                            .push_quote_line("• https://shimmer.network")
											.push_quote_line("• https://wiki.iota.org");
                                            let mut thread_msg = thread
                                                .send_message(&ctx.http, |t| {
                                                    t.content(prepared_msg)
                                                })
                                                .await
                                                .unwrap();
                                            thread_msg.suppress_embeds(&ctx.http).await.unwrap();
                                            // } else {
                                            // 	let warn_msg = msg
                                            // 	.reply_mention(
                                            // 		&ctx.http,
                                            // 		"Please reply in threads above instead of here",
                                            // 	)
                                            // 	.await
                                            // 	.unwrap();
                                            // 	sleep(Duration::from_secs(10)).await;
                                            // 	warn_msg.delete(&ctx.http).await.unwrap();
                                            // 	msg.delete(&ctx.http).await.ok();
                                            // }
                                        }
                                        // }
                                    }
                                }

                                // save the found from data
                                followup_results
                                    .data
                                    .values
                                    .iter()
                                    .for_each(|x| role_choices.push(x.to_string()));

                                // Remove old roles
                                if let Some(roles) = member.roles(&ctx.cache) {
                                    // Remove all assignable roles first
                                    let mut all_assignable_roles: Vec<SelectMenuSpec> = Vec::new();
                                    all_assignable_roles.append(&mut additional_roles);
                                    all_assignable_roles.append(&mut poll_entries);
                                    let mut removeable_roles: Vec<RoleId> = Vec::new();

                                    for role in roles {
                                        if all_assignable_roles.iter().any(|x| x.value == role.name)
                                        {
                                            removeable_roles.push(role.id);
                                        }
                                    }
                                    if !removeable_roles.is_empty() {
                                        member
                                            .remove_roles(&ctx.http, &removeable_roles)
                                            .await
                                            .unwrap();
                                    }
                                }

                                assign_roles(
                                    &mci,
                                    ctx,
                                    role_choices,
                                    &mut member,
                                    &temp_role,
                                    &member_role,
                                )
                                .await;
                                break;
                            }
                            _ => {}
                        }
                    }
                }
                _ => {
                    // If a Question thread suggestion was clicked
                    if mci.data.custom_id.starts_with("http") {
                        let button_label = &mci
                            .message
                            .components
                            .iter()
                            .find_map(|a| {
                                a.components.iter().find_map(|x| {
                                    let button: Button =
                                        serde_json::from_value(serde_json::to_value(x).unwrap())
                                            .unwrap();
                                    if button.custom_id.unwrap() == mci.data.custom_id {
                                        Some(button.label.unwrap())
                                    } else {
                                        None
                                    }
                                })
                            })
                            .unwrap();

                        mci.create_interaction_response(&ctx.http, |r| {
                            r.kind(InteractionResponseType::ChannelMessageWithSource)
                                .interaction_response_data(|d| {
                                    d.content(format!("{}: {button_label}", &mci.user.mention()))
                                        .components(|c| {
                                            c.create_action_row(|a| {
                                                a.create_button(|b| {
                                                    b.label("Open link")
                                                        .url(&mci.data.custom_id)
                                                        .style(ButtonStyle::Link)
                                                })
                                            })
                                        })
                                        .flags(MessageFlags::EPHEMERAL)
                                })
                        })
                        .await
                        .unwrap();

                        mci.message
                            .react(&ctx.http, ReactionType::Unicode("🔎".to_string()))
                            .await
                            .unwrap();
                    }
                }
            }
        }
        Interaction::ApplicationCommand(mci) => match mci.data.name.as_str() {
            "close" => {
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
            "nothing_to_see_here" => {
                let input = mci
                    .data
                    .options
                    .get(0)
                    .expect("Expected input")
                    .value
                    .as_ref()
                    .unwrap();
                mci.create_interaction_response(&ctx.http, |r| {
                    r.kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|d| {
                            d.content("Posted message on this channel")
                                .flags(MessageFlags::EPHEMERAL)
                        })
                })
                .await
                .unwrap();

                mci.channel_id
                    .send_message(&ctx.http, |m| {
                        m.content(
                            input
                                .to_string()
                                .trim_start_matches('"')
                                .trim_end_matches('"'),
                        )
                    })
                    .await
                    .unwrap();
            }
            _ => {}
        },
        Interaction::ModalSubmit(mci) => {
            let typing = mci.channel_id.start_typing(&ctx.http).unwrap();
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

            let img_url = reqwest::Url::parse(&mci.user.face().replace(".webp", ".png")).unwrap();
            let webhook = mci
                .channel_id
                .create_webhook_with_avatar(&ctx, &user_name, AttachmentType::Image(img_url))
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
            typing.stop().unwrap();
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
                                    .style(ButtonStyle::Danger)
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
                let mut prefix_emojis: HashMap<&str, Emoji> = HashMap::new();
                let emoji_sources: HashMap<&str, &str> = HashMap::from([
					("gitpod", "https://www.gitpod.io/images/media-kit/logo-mark.png"),
					("github", "https://cdn.discordapp.com/attachments/981191970024210462/981192908780736573/github-transparent.png"),
					("discord", "https://discord.com/assets/9f6f9cd156ce35e2d94c0e62e3eff462.png")
				]);
                let guild = &mci.guild_id.unwrap();
                for source in ["gitpod", "github", "discord"].iter() {
                    let emoji = {
                        if let Some(emoji) = guild
                            .emojis(&ctx.http)
                            .await
                            .unwrap()
                            .into_iter()
                            .find(|x| x.name == *source)
                        {
                            emoji
                        } else {
                            let dw_path = env::current_dir().unwrap().join(format!("{source}.png"));
                            let dw_url = emoji_sources.get(source).unwrap().to_string();
                            let client = reqwest::Client::new();
                            let downloaded_bytes = client
                                .get(dw_url)
                                .timeout(Duration::from_secs(5))
                                .send()
                                .await
                                .unwrap()
                                .bytes()
                                .await
                                .unwrap();
                            tokio::fs::write(&dw_path, &downloaded_bytes).await.unwrap();
                            let emoji_image = read_image(dw_path).unwrap();
                            let emoji_image = emoji_image.as_str();
                            guild
                                .create_emoji(&ctx.http, source, emoji_image)
                                .await
                                .unwrap()
                        }
                    };
                    prefix_emojis.insert(source, emoji);
                }

                let mut suggested_count = 1;
                thread.send_message(&ctx.http, |m| {
				m.content(format!("{} I also found some relevant links which might answer your question, please do check them out below 🙏:", &user_mention));
					m.components(|c| {
						loop {
							if suggested_count > 10 || relevant_links.is_empty() {
								break;
							}
							c.create_action_row(|a|
								{
									let mut i = 1;
									for (title, url) in relevant_links.clone() {
										if i > 5 {
											break;
										} else {
											i += 1;
											relevant_links.remove(&title);
										}
										let emoji = {
											if url.starts_with("https://www.gitpod.io") {
												prefix_emojis.get("gitpod").unwrap()
											} else if url.starts_with("https://github.com") {
												prefix_emojis.get("github").unwrap()
											} else {
												prefix_emojis.get("discord").unwrap()
											}
										};

										a.create_button(|b|b.label(&title.as_str().substring(0, 80)).custom_id(&url.as_str().substring(0, 100)).style(ButtonStyle::Secondary).emoji(ReactionType::Custom {
											id: emoji.id,
											name: Some(emoji.name.clone()),
											animated: false,
										}));
									}
										a
									}
								);
								suggested_count += 1;
						}
							c
						});
						m
					}
				).await.unwrap();
                thread_typing.stop().unwrap();
            }
            // if !relevant_links.is_empty() {
            //     thread
            //         .send_message(&ctx.http, |m| 
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

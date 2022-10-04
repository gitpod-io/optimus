use super::*;
use crate::db::ClientContextExt;

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
        if thread_node.name.contains('✅') || thread_node.name.contains('❓') {
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

async fn show_issue_form(mci: &MessageComponentInteraction, ctx: &Context) {
    // let db = &ctx.get_db().await;
    // let desc = {
    //     if let Ok(result) = db
    //         .get_pending_question_content(&mci.user.id, &mci.channel_id)
    //         .await
    //     {
    //         db.remove_pending_question(&mci.user.id, &mci.channel_id)
    //             .await
    //             .ok();
    //         result
    //     } else {
    //         "".to_string()
    //     }
    // };

    let parent_channel_id = &mci
        .channel_id
        .to_channel(&ctx.http)
        .await
        .unwrap()
        .guild()
        .unwrap()
        .parent_id
        .unwrap();

    // Temp
    mci.create_interaction_response(&ctx.http, |r| {
        r.kind(InteractionResponseType::ChannelMessageWithSource);
        r.interaction_response_data(|d| {
            let mut msg = MessageBuilder::new();
            msg.push_quote_line(format!(
                "{} **{}**",
                &mci.user.mention(),
                "Please share the following (if applies):"
            ));

            if *parent_channel_id != SELFHOSTED_QUESTIONS_CHANNEL {
                msg.push_line("• Contents of your `.gitpod.yml`")
                    .push_line("• Contents of your `.gitpod.Dockerfile")
                    .push_line("• An example repository link");
                d.content(msg.build());
            } else {
                msg.push_line("• Contents of your `config.yml`")
                    .push_line("• Result of:\n```bash\nkubectl get pods -n <namespace>\n```");
                d.content(msg.build());
            }

            d
        });
        r
    })
    .await
    .unwrap();

    return;
    // There is a discord UI bug with it.
    // Not going to execute below code until https://github.com/discord/discord-api-docs/issues/5302 is fixed :(
    mci.create_interaction_response(&ctx, |r| {
        r.kind(InteractionResponseType::Modal);
        r.interaction_response_data(|d| {
            d.custom_id("gitpod_help_button_press");
            d.title("Template");
            d.components(|c| {
                // c.create_action_row(|ar| {
                //     ar.create_input_text(|it| {
                //         it.style(InputTextStyle::Short)
                //             .custom_id("input_title")
                //             .required(true)
                //             .label("Title")
                //             .max_length(98)
                //     })
                // });
                // c.create_action_row(|ar| {
                //     ar.create_input_text(|it| {
                //         it.style(InputTextStyle::Paragraph)
                //             .custom_id("input_description")
                //             .label("Description")
                //             .required(true)
                //             .max_length(4000)
                //             // .value(desc)
                //     })
                // });
                c.create_action_row(|ar| {
                    ar.create_input_text(|it| {
                        if *parent_channel_id != SELFHOSTED_QUESTIONS_CHANNEL {
                            it.style(InputTextStyle::Paragraph)
                                .custom_id("input_gitpod_yml")
                                .label("Your .gitpod.yml contents")
                                .required(false)
                            // .max_length(4000)
                        } else {
                            it.style(InputTextStyle::Paragraph)
                                .custom_id("input_config_yaml")
                                .label("Your config.yaml contents")
                                .required(false)
                            // .max_length(1000)
                        }
                    })
                });
                c.create_action_row(|ar| {
                    ar.create_input_text(|it| {
                        if *parent_channel_id != SELFHOSTED_QUESTIONS_CHANNEL {
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
                                .value(SELFHOSTED_KUBECTL_COMMAND_PLACEHOLDER)
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
            match mci.data.custom_id.as_str() {
                "gitpod_complete_question_submit" => show_issue_form(&mci, ctx).await,
                "gitpod_close_issue" => close_issue(&mci, ctx).await,
                "getting_started_letsgo" => {
                    let mut additional_roles: Vec<SelectMenuSpec> = Vec::from([
                        SelectMenuSpec {
                            value: "JetBrainsIDEs",
                            description: "Discuss about Jetbrains IDEs for Gitpod!",
                            label: "JetBrains (BETA)",
                            display_emoji: "🧠",
                        },
                        SelectMenuSpec {
                            value: "DevX",
                            description: "All things about DevX",
                            label: "Developer Experience",
                            display_emoji: "✨",
                        },
                        SelectMenuSpec {
                            value: "SelfHosted",
                            description: "Do you selfhost Gitpod? Then you need this!",
                            label: "Self Hosted Gitpod",
                            display_emoji: "🏡",
                        },
                        SelectMenuSpec {
                            value: "OnMobile",
                            description: "Talk about using Gitpod on mobile devices",
                            label: "Mobile and tablets",
                            display_emoji: "📱",
                        },
                    ]);

                    let mut poll_entries: Vec<SelectMenuSpec> = Vec::from([
                        SelectMenuSpec {
                            value: "Found: FromFriend",
                            label: "Friend or colleague",
                            description: "A friend or colleague of mine introduced Gitpod to me",
                            display_emoji: "🫂",
                        },
                        SelectMenuSpec {
                            value: "Found: FromGoogle",
                            label: "Google",
                            description: "I found Gitpod from a Google search",
                            display_emoji: "🔎",
                        },
                        SelectMenuSpec {
                            value: "Found: FromYouTube",
                            label: "YouTube",
                            description: "Saw Gitpod on a Youtube Video",
                            display_emoji: "📺",
                        },
                        SelectMenuSpec {
                            value: "Found: FromTwitter",
                            label: "Twitter",
                            description: "Saw people talking about Gitpod on a Tweet",
                            display_emoji: "🐦",
                        },
                        SelectMenuSpec {
                            value: "Found: FromGitRepo",
                            label: "Git Repository",
                            description: "Found Gitpod on a Git repository",
                            display_emoji: "✨",
                        },
                    ]);

                    for prog_role in [
                        "Bash", "C", "CPP", "CSharp", "Docker", "Go", "Haskell", "Java", "Js",
                        "Kotlin", "Lua", "Nim", "Nix", "Node", "Perl", "Php", "Python", "Ruby",
                        "Rust",
                    ]
                    .iter()
                    {
                        additional_roles.push(SelectMenuSpec {
                            label: prog_role,
                            description: "Discussions",
                            display_emoji: "📜",
                            value: prog_role,
                        });
                    }
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
                                    s.custom_id("channel_choice").max_values(24)
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
													b.label("To get help with Gitpod.io");
													b.style(ButtonStyle::Secondary);
													b.emoji(ReactionType::Unicode("✌️".to_string()));
													b.custom_id("gitpodio_help")
												});
												a.create_button(|b|{
													b.label("To get help with my selfhosted installation");
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
                                    label: "Subscribed",
                                    description: "Subscribed to pings",
                                    display_emoji: "",
                                    value: "Subscriber",
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

                                let temp_role = get_role(&mci, ctx, "Temp").await;
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
												.push_quote_line("🔧 what you’re working on!")
												.push_quote_line("🛑 what blocks you most in your daily dev workflow")
												.push_quote_line("🌈 your favourite Gitpod feature")
												.push_quote_line("✨ your favourite emoji").build()
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

                                            if words_count::count(&msg.content).words > 4 {
                                                for unicode in ["👋", "🔥"] {
                                                    msg.react(
                                                        &ctx.http,
                                                        ReactionType::Unicode(unicode.to_string()),
                                                    )
                                                    .await
                                                    .unwrap();
                                                }
                                            } else {
                                                msg.delete(&ctx.http).await.unwrap();
                                            }

                                            let general_channel = if cfg!(debug_assertions) {
                                                ChannelId(947769443516284943)
                                            } else {
                                                ChannelId(839379835662368768)
                                            };
                                            let offtopic_channel = if cfg!(debug_assertions) {
                                                ChannelId(947769443793141769)
                                            } else {
                                                ChannelId(972510491933032508)
                                            };

                                            let mut prepared_msg = MessageBuilder::new();
                                            prepared_msg.push_line(format!(
                                                "Welcome to the Gitpod community {} 🙌\n",
                                                &msg.author.mention()
                                            ));
                                            match join_reason.as_str() {
                                                "gitpodio_help" => {
                                                    prepared_msg.push_line(
														format!("**You mentioned that** you need help with Gitpod.io, please ask in {}\n",
																	&QUESTIONS_CHANNEL.mention())
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
																	&SELFHOSTED_QUESTIONS_CHANNEL.mention())
													);
                                                }
                                                _ => {}
                                            }
                                            prepared_msg.push_bold_line("Here are some channels that you should check out:")
											.push_quote_line(format!("• {} - for tech, programming and anything related 🖥", &general_channel.mention()))
											.push_quote_line(format!("• {} - for any random discussions ☕️", &offtopic_channel.mention()))
											.push_quote_line(format!("• {} - have a question about Gitpod? this is the place to ask! ❓\n", &QUESTIONS_CHANNEL.mention()))
											.push_line("…And there’s more! Take your time to explore :)\n")
											.push_bold_line("Feel free to check out the following pages to learn more about Gitpod:")
											.push_quote_line("• https://www.gitpod.io/community")
											.push_quote_line("• https://www.gitpod.io/about");
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
                                    // .flags(MessageFlags::EPHEMERAL)
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
                let thread_node = mci
                    .channel_id
                    .to_channel(&ctx.http)
                    .await
                    .unwrap()
                    .guild()
                    .unwrap();
                let thread_type = {
                    if thread_node.name.contains('✅') || thread_node.name.contains('❓') {
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
            let optional_one = match mci
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
            let optional_two = match mci
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

            let parent_channel_id = &mci
                .channel_id
                .to_channel(&ctx.http)
                .await
                .unwrap()
                .guild()
                .unwrap()
                .parent_id
                .unwrap();
            // let self_avatar = &ctx.cache.current_user().await.face();
            // let self_name = &ctx.cache.current_user().await.name;

            // if mci.data.custom_id == "gitpod_help_button_press" {
            //     if let Some(msg) = mci.message {
            //         msg.delete(&ctx.http).await.ok();
            //     }
            // }

            let user_mention = mci.user.mention();
            let thread = mci.channel_id;

            thread
                .send_message(&ctx.http, |m| {
                    if *parent_channel_id != SELFHOSTED_QUESTIONS_CHANNEL {
                        if !optional_one.value.is_empty() || !optional_two.value.is_empty() {
                            m.add_embed(|e| {
                                if !optional_one.value.is_empty() {
                                    e.field(".gitpod.yml contents", &optional_one.value, false);
                                }
                                if !optional_two.value.is_empty() {
                                    e.field("Example Repository", &optional_two.value, false);
                                }
                                e
                            });
                        }
                    } else if *parent_channel_id == SELFHOSTED_QUESTIONS_CHANNEL {
                        if !optional_one.value.is_empty() {
                            m.add_embed(|e| {
                                e.title("config.yaml contents")
                                    .description(format!("```yaml\n{}\n```", &optional_one.value))
                            });
                        }
                        if optional_two.value != SELFHOSTED_KUBECTL_COMMAND_PLACEHOLDER
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

            // questions_thread::responder(ctx).await;
        }
        _ => (),
    }
}

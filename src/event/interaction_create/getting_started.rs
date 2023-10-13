use std::time::Duration;

use anyhow::{bail, Context as _, Result};
use serenity::{
    client::Context,
    futures::StreamExt,
    model::{
        application::{
            component::ButtonStyle,
            interaction::{
                message_component::MessageComponentInteraction, InteractionResponseType,
                MessageFlags,
            },
        },
        channel::ReactionType,
        guild::Member,
        guild::Role,
        id::RoleId,
        prelude::*,
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

async fn get_role(mci: &MessageComponentInteraction, ctx: &Context, name: &str) -> Result<Role> {
    let guild_id = mci.guild_id.context("Ok")?;
    let role = {
        if let Some(result) = guild_id
            .to_guild_cached(&ctx.cache)
            .context("Failed to get guild ID")?
            .role_by_name(name)
        {
            result.clone()
        } else {
            let r = guild_id
                .create_role(&ctx.http, |r| {
                    r.name(name);
                    r.mentionable(false);
                    r.hoist(false);
                    r
                })
                .await?;

            r.clone()
        }
    };

    if role.name != "Member" && role.name != "Gitpodders" && !role.permissions.is_empty() {
        role.edit(&ctx.http, |r| r.permissions(Permissions::empty()))
            .await?;
    }

    Ok(role)
}

async fn assign_roles(
    mci: &MessageComponentInteraction,
    ctx: &Context,
    role_choices: Vec<String>,
    member: &mut Member,
    temp_role: &Role,
    member_role: &Role,
) -> Result<()> {
    if role_choices.len() > 1 || !role_choices.iter().any(|x| x == "none") {
        // Is bigger than a single choice or doesnt contain none

        let mut role_ids: Vec<RoleId> = Vec::new();
        for role_name in role_choices {
            if role_name == "none" {
                continue;
            }
            let role = get_role(mci, ctx, role_name.as_str()).await.context("ok")?;
            role_ids.push(role.id);
        }
        member.add_roles(&ctx.http, &role_ids).await?;
    }

    // Remove the temp role from user
    if member.roles.iter().any(|x| x == &temp_role.id) {
        member.remove_role(&ctx.http, temp_role.id).await?;
    }
    // Add member role if missing
    if !member.roles.iter().any(|x| x == &member_role.id) {
        member.add_role(&ctx.http, member_role.id).await?;
    }

    Ok(())
}

pub async fn responder(mci: &MessageComponentInteraction, ctx: &Context) -> Result<()> {
    let config = crate::BOT_CONFIG.get().context("Failed to get BotConfig")?;

    let channels = config
        .discord
        .channels
        .as_ref()
        .context("No discord channels defined")?;

    let primary_questions_channel = channels
        .primary_questions_channel_id
        .as_ref()
        .context("No primary channel found")?;
    let secondary_questions_channel = channels
        .secondary_questions_channel_id
        .as_ref()
        .context("No secondary channel found")?;
    let introduction_channel = channels
        .introduction_channel_id
        .as_ref()
        .context("No introduction channel found")?;
    let general_channel = channels
        .general_channel_id
        .as_ref()
        .context("No general channel found")?;
    let off_topic_channel = channels
        .off_topic_channel_id
        .as_ref()
        .context("No off topic channel found")?;

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

    // Add more Roles related with Programming to additional_roles array
    for prog_role in [
        "Bash", "C", "CPP", "CSharp", "Docker", "Go", "Haskell", "Java", "Js", "Kotlin", "Lua",
        "Nim", "Nix", "Node", "Perl", "Php", "Python", "Ruby", "Rust",
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

    // User inputs
    let mut role_choices: Vec<String> = Vec::new();
    let mut join_reason = String::new();

    mci.create_interaction_response(&ctx.http, |r| {
        r.kind(InteractionResponseType::ChannelMessageWithSource);
        r.interaction_response_data(|d| {
            d.content("**[1/5]:** Which additional channels would you like to have access to?");
            d.components(|c| {
                c.create_action_row(|a| {
                    a.create_select_menu(|s| {
                        s.placeholder("Select channels (Optional)");
                        s.options(|o| {
                            for spec in &additional_roles {
                                o.create_option(|opt| {
                                    opt.label(spec.label);
                                    opt.description(spec.description);
                                    opt.emoji(ReactionType::Unicode(
                                        spec.display_emoji.to_string(),
                                    ));
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
            d.custom_id("additional_roles")
                .flags(MessageFlags::EPHEMERAL)
        });
        r
    })
    .await?;

    let mut interactions = mci
        .get_interaction_response(&ctx.http)
        .await?
        .await_component_interactions(ctx)
        .timeout(Duration::from_secs(60 * 5))
        .build();

    while let Some(interaction) = interactions.next().await {
        match interaction.data.custom_id.as_str() {
            "channel_choice" => {
                interaction.create_interaction_response(&ctx.http, |r| {
                    r.kind(InteractionResponseType::UpdateMessage).interaction_response_data(|d|{
                        d.content("**[2/5]:** Would you like to get notified for announcements and community events?");
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
                }).await?;

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
										d.content("**[3/5]:** Why did you join our community?\nI will point you to the correct channels with this info.").components(|c| {
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
								}).await?;

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
                interaction
                    .create_interaction_response(&ctx.http, |r| {
                        r.kind(InteractionResponseType::Modal)
                            .interaction_response_data(|d| {
                                d.custom_id("company_name_submitted")
                                    .title("[4/5] Share your company name")
                                    .components(|c| {
                                        c.create_action_row(|a| {
                                            a.create_input_text(|t| {
                                                t.custom_id("company_submitted")
                                                    .label("Company (optional)")
                                                    .placeholder("Type in your company name")
                                                    .max_length(50)
                                                    .min_length(2)
                                                    .required(false)
                                                    .style(component::InputTextStyle::Short)
                                            })
                                        })
                                    })
                            })
                    })
                    .await?;

                // Save join reason
                join_reason.push_str(interaction.data.custom_id.as_str());

                // Fetch member
                let mut member = mci.member.clone().context("Can't fetch member")?;
                let member_role = get_role(mci, ctx, "Member").await?;
                let never_introduced = {
                    let mut status = true;
                    if let Some(roles) = member.roles(&ctx.cache) {
                        let gitpodder_role = get_role(mci, ctx, "Gitpodders").await?;
                        status = !roles
                            .into_iter()
                            .any(|x| x == member_role || x == gitpodder_role);
                    }
                    if status {
                        let mut count = 0;
                        if let Ok(intro_msgs) = &ctx
                            .http
                            .get_messages(*introduction_channel.as_u64(), "")
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
                        d.content("**[5/5]:** How did you find Gitpod?");
                        d.components(|c| {
                            c.create_action_row(|a| {
                                a.create_select_menu(|s| {
                                    s.placeholder("[Poll]: Select sources (Optional)");
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
                                                .emoji(ReactionType::Unicode("⏭".to_string()));
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
                    .await?;

                let temp_role = get_role(mci, ctx, "Temp").await?;
                let followup_results =
                    match followup
                        .await_component_interaction(ctx)
                        .timeout(Duration::from_secs(60 * 5))
                        .await
                    {
                        Some(ci) => {
                            member.add_role(&ctx.http, temp_role.id).await?;
                            let final_msg =
                                {
                                    if never_introduced {
                                        MessageBuilder::new()
												.push_line(format!(
													"Thank you {}! To unlock the server, drop by {} :wave:",
													interaction.user.mention(),
													introduction_channel.mention()
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
                            .await?;
                            ci
                        }
                        None => bail!("Did not interact in time"),
                    };

                // if let Some(interaction) = interaction
                //     .get_interaction_response(&ctx.http)
                //     .await
                //     ?
                //     .await_component_interaction(&ctx)
                //     .timeout(Duration::from_secs(60 * 5))
                //     .await
                // {

                if never_introduced {
                    // Wait for the submittion on INTRODUCTION_CHANNEL
                    if let Some(msg) = mci
                        .user
                        .await_reply(ctx)
                        .timeout(Duration::from_secs(60 * 30))
                        .await
                    {
                        // Watch intro channel
                        if &msg.channel_id == introduction_channel {
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
                                    t.auto_archive_duration(1440)
                                        .name(format!("Welcome {}!", msg.author.name))
                                })
                                .await?;

                            if words_count::count(&msg.content).words > 4 {
                                for unicode in ["👋", "🔥"] {
                                    msg.react(
                                        &ctx.http,
                                        ReactionType::Unicode(unicode.to_string()),
                                    )
                                    .await?;
                                }
                            } else {
                                msg.delete(&ctx.http).await?;
                            }

                            let mut prepared_msg = MessageBuilder::new();
                            prepared_msg.push_line(format!(
                                "Welcome to the Gitpod community {} 🙌\n",
                                &msg.author.mention()
                            ));
                            match join_reason.as_str() {
                                "gitpodio_help" => {
                                    prepared_msg.push_line(
														format!("**You mentioned that** you need help with Gitpod.io, please ask in {}\n",
																	&primary_questions_channel.mention())
													);
                                }
                                "selfhosted_help" => {
                                    let selfhosted_role = get_role(mci, ctx, "SelfHosted").await?;
                                    member.add_role(&ctx.http, selfhosted_role.id).await?;
                                    prepared_msg.push_line(
														format!("**You mentioned that** you need help with selfhosted, please ask in {}\n",
																	&secondary_questions_channel.mention())
													);
                                }
                                _ => {}
                            }
                            prepared_msg.push_bold_line("Here are some channels that you should check out:")
											.push_quote_line(format!("• {} - for tech, programming and anything related 🖥", &general_channel.mention()))
											.push_quote_line(format!("• {} - for any random discussions ☕️", &off_topic_channel.mention()))
											.push_quote_line(format!("• {} - have a question about Gitpod? this is the place to ask! ❓\n", &primary_questions_channel.mention()))
											.push_line("…And there’s more! Take your time to explore :)\n")
											.push_bold_line("Feel free to check out the following pages to learn more about Gitpod:")
											.push_quote_line("• https://www.gitpod.io/community")
											.push_quote_line("• https://www.gitpod.io/about");
                            let mut thread_msg = thread
                                .send_message(&ctx.http, |t| t.content(prepared_msg))
                                .await?;
                            thread_msg.suppress_embeds(&ctx.http).await?;
                            // } else {
                            // 	let warn_msg = msg
                            // 	.reply_mention(
                            // 		&ctx.http,
                            // 		"Please reply in threads above instead of here",
                            // 	)
                            // 	.await
                            // 	?;
                            // 	sleep(Duration::from_secs(10)).await;
                            // 	warn_msg.delete(&ctx.http).await?;
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
                        if all_assignable_roles.iter().any(|x| x.value == role.name) {
                            removeable_roles.push(role.id);
                        }
                    }
                    if !removeable_roles.is_empty() {
                        member.remove_roles(&ctx.http, &removeable_roles).await?;
                    }
                }

                assign_roles(
                    mci,
                    ctx,
                    role_choices,
                    &mut member,
                    &temp_role,
                    &member_role,
                )
                .await?;

                break;
            }
            _ => {}
        }
    }

    Ok(())
}

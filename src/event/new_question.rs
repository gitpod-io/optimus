use crate::{init::MEILICLIENT_THREAD_INDEX, utils::index_threads::Thread};
use url::Url;

use super::substr::StringUtils;
use color_eyre::eyre::{eyre, Report, Result};

use regex::Regex;
use serenity::{
    client::Context,
    model::{application::component::ButtonStyle, channel::ReactionType},
    prelude::Mentionable,
};
use serenity::{
    model::{guild::Emoji, prelude::Message},
    utils::{read_image, MessageBuilder},
};
use std::{collections::HashMap, env, time::Duration, iter::repeat_with};
use tokio::time::sleep;
use urlencoding::encode;

async fn save_and_fetch_links(
    sites: &[&str],
    title: &str,
    _description: &str,
) -> Option<HashMap<String, String>> {
    let mut links: HashMap<String, String> = HashMap::new();

    let client = reqwest::Client::new();

    // Fetch matching links
    for site in sites.iter() {
        if let Ok(resp) = client
        .get(
                format!(
                    "https://www.google.com/search?q=site:{} {}",
                    encode(site),
                    encode(title)
                )
            )
        .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.182 Safari/537.36")
        .send()
        .await {
            if let Ok(result) = resp.text().await {

                for (i, caps) in
                    // [^:~] avoids the google hyperlinks
                    Regex::new(format!("\"(?P<url>{}/.[^:~]*?)\"", &site).as_str())
                        .unwrap()
                        .captures_iter(&result).enumerate()
                {
                    // 3 MAX each, starts at 0
                    if i == 3 {
                        break;
                    }

                    let url = &caps["url"];
                    let captured_hash = Regex::new(r"(?P<hash>#[^:~].*)").ok()?
                            .captures(url)
                            .and_then(|cap| {
                                cap.name("hash")
                                .map(|name| name.as_str())
                            });

                    if let Ok(resp) = client.get(url)
                            .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.182 Safari/537.36")
                    .send()
                    .await {
                        if let Ok(result) = resp.text().await {
                            let result = html_escape::decode_html_entities(&result).to_string();
                            for caps in Regex::new(r"<title>(?P<title>.*?)</title>").unwrap().captures_iter(&result) {
                                let title = &caps["title"];

                                let text = if let Some(hash) = captured_hash {
                                    format!("{} | {}", title, hash)
                                } else {
                                    title.to_string()
                                };
                                //links.push_str(format!("• __{}__\n\n", text).as_str());
                                links.insert(text, url.to_string());
                            }
                        }
                    }

                }
            }
        }
    }

    // // Fetch 5 MAX matching discord questions
    if let Some(mclient) = MEILICLIENT_THREAD_INDEX.get()
    && let Ok(data) = mclient.search().with_query(title).with_limit(5).execute::<Thread>().await {
        for ids in data.hits {
            links.insert(
                ids.result.title,
                format!(
                    "https://discord.com/channels/{}/{}",
                    ids.result.guild_id, ids.result.id
                ),
            );
        }
    }

    Some(links)
}

pub async fn responder(ctx: &Context, msg: &Message) -> Result<(), Report> {
    // Config
    if let Some(config) = crate::BOT_CONFIG.get() && let Some(channels) = &config.discord.channels
    && let Some(primary_questions_channel) = channels.primary_questions_channel_id
    && let Some(secondary_questions_channel) = channels.secondary_questions_channel_id 
    // Check if thread
    && let Some(thread) = msg.channel(&ctx.http).await?.guild()
    && let Some(parent_channel_id) = thread.parent_id
    && [primary_questions_channel, secondary_questions_channel].contains(&parent_channel_id)
    && msg.id.as_u64() == thread.id.as_u64() {

        let user_mention = &msg.author.mention();
        let user_without_mention = &msg.author.name;

        // Message node
        thread
        .send_message(&ctx, |message| {
            message.content(
                MessageBuilder::new()
                    .push_quote(
                        format!(
                            "Hey {}! Thank you for raising this — please hang tight as someone from our community may help you out.",
                            &user_without_mention
                        )
                    ).build()
            );

            // Buttons
            message.components(|c| {
                c.create_action_row(|ar| {
                    ar.create_button(|button| {
                        button
                            .style(ButtonStyle::Danger)
                            .label("Close")
                            .custom_id("gitpod_close_issue")
                            .emoji(ReactionType::Unicode("🔒".to_string()))
                    });
                    ar.create_button(|button| {
                        button
                            // .custom_id("gitpod_docs_link")
                            .style(ButtonStyle::Link)
                            .label("Docs")
                            .emoji(ReactionType::Unicode("📚".to_string()))
                            .url("https://www.gitpod.io/docs/")
                    });
                    ar.create_button(|button| {
                        button.style(ButtonStyle::Link).label("YouTube").url(
                            "https://youtube.com/playlist?list=PL3TSF5whlprXVp-7Br2oKwQgU4bji1S7H",
                        ).emoji(ReactionType::Unicode("📺".to_string()))
                    });
                    // Final return
                    ar.create_button(|button| {
                        button
                            .style(ButtonStyle::Link)
                            .label("Status")
                            .emoji(ReactionType::Unicode("🧭".to_string()))
                            .url("https://www.gitpodstatus.com/")
                    })
                })
            });

            message

        })
        .await?;

        // Simulate typing
        let thread_typing = thread.clone().start_typing(&ctx.http)?;

        // Fetch suggestions from relevant sources
        let relevant_links_external_sources = {
            if parent_channel_id != secondary_questions_channel {
                Vec::from(["https://www.gitpod.io/docs", "https://github.com/gitpod-io"])
            } else {
                Vec::from(["https://github.com/gitpod-io"])
            }
        };

        if let Some(mut relevant_links) = save_and_fetch_links(
            &relevant_links_external_sources,
            &thread.name,
            &msg.content,
        ).await {

            let mut prefix_emojis: HashMap<&str, Emoji> = HashMap::new();
            let emoji_sources: HashMap<&str, &str> = HashMap::from([
                ("gitpod", "https://www.gitpod.io/images/media-kit/logo-mark.png"),
                ("github", "https://cdn.discordapp.com/attachments/981191970024210462/981192908780736573/github-transparent.png"),
                ("discord", "https://discord.com/assets/9f6f9cd156ce35e2d94c0e62e3eff462.png")
            ]);
            let guild = &msg.guild_id.ok_or_else(||eyre!("Failed to get GuildId"))?;
            for source in ["gitpod", "github", "discord"].iter() {
                let emoji = {
                    if let Some(emoji) = guild
                        .emojis(&ctx.http)
                        .await?
                        .into_iter()
                        .find(|x| x.name == *source)
                    {
                        emoji
                    } else {
                        let dw_path =
                            env::current_dir()?.join(format!("{source}.png"));
                        let dw_url = emoji_sources.get(source)
                            .ok_or_else(||eyre!("Emoji source {source} doesn't exist"))?
                            .to_string();
                        let client = reqwest::Client::new();
                        let downloaded_bytes = client
                            .get(dw_url)
                            .timeout(Duration::from_secs(5))
                            .send()
                            .await?
                            .bytes()
                            .await?;
                        tokio::fs::write(&dw_path, &downloaded_bytes).await?;
                        let emoji_image = read_image(dw_path)?;
                        let emoji_image = emoji_image.as_str();
                        guild
                            .create_emoji(&ctx.http, source, emoji_image)
                            .await?
                    }
                };
                prefix_emojis.insert(source, emoji);
            }

            let mut suggested_block_count = 0;
            thread.send_message(&ctx.http, |m| {
                m.content(format!("{}, I found some relevant links which might help to self-serve, please do check them out below 🙏:", &user_without_mention));
                m.components(|c| {
                    // TODO: We could use a more concise `for` loop, but anyway
                    loop {
                        // 2 suggestion blocks MAX, means ~10 links
                        if suggested_block_count == 2 || relevant_links.is_empty() {
                            break;
                        } else {
                            suggested_block_count += 1;
                        }

                        c.create_action_row(|a| {
                            for (i, (title, url)) in relevant_links.clone().iter().enumerate() {
                                // 5 MAX, starts at 0
                                if i == 5 {
                                    break;
                                } else {
                                    relevant_links.remove(title);
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

                                if let Ok(mut parsed_url) = Url::parse(url) {

                                    let random_str: String = repeat_with(fastrand::alphanumeric).take(4).collect();
                                    parsed_url.query_pairs_mut().append_key_only(&random_str);

                                    a.create_button(|b| b.label(title.as_str().substring(0, 80))
                                        .custom_id(parsed_url.as_str().substring(0, 100))
                                        .style(ButtonStyle::Secondary)
                                        .emoji(ReactionType::Custom {
                                            id: emoji.id,
                                            name: Some(emoji.name.clone()),
                                            animated: false,
                                        })
                                    );
                                }
                                    // .query_pairs_mut()
                                    // .append_key_only(&random_str)
                                    // .finish();

                            }
                                a
                        });
                    }
                        c
                    });
                m
            }).await?;
        }

        // Take a pause
        sleep(Duration::from_secs(5)).await;

        let mut msg = MessageBuilder::new();
        // Ask for info
        msg.push_quote_line(format!(
            "{} **{}**",
            &user_mention, "You can share the following (if applies):"
        ));

        if parent_channel_id != secondary_questions_channel {
            msg.push_line("• Contents of your `.gitpod.yml`")
                .push_line("• Contents of your `.gitpod.Dockerfile`")
                .push_line("• An example repository link");
        } else {
            msg.push_line("• Contents of your `config.yml`")
            .push("• Result of:```bash\nkubectl get pods -n <namespace>```")
            .push_line("• If you have resources that are set up strangely, please run `kubectl describe` on the resource");
        }

        // AI prompt
        msg.push_line("\n> ✨ **NEW:** Try our experimental Gitpod Docs AI!");

        thread
            .send_message(&ctx.http, |message| {
                message.content(msg.build());
                
                message.components(|c| {
                    c.create_action_row(|ar| {
                        ar.create_button(|button| {
                            button
                                .style(ButtonStyle::Primary)
                                .label("Ask Gitpod Docs AI")
                                .custom_id("question-qa")
                                .emoji(ReactionType::Unicode("🔍".to_string()))
                        })
                    })
                });
                message
            })
            .await?;

        thread_typing.stop().ok_or_else(|| eyre!("Couldn't stop writing"))?;
    };

    Ok(())
}

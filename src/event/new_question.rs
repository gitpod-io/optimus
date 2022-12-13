use super::substr::StringUtils;
use super::{QUESTIONS_CHANNEL, SELFHOSTED_QUESTIONS_CHANNEL};
use anyhow::Result;
use meilisearch_sdk::{client::Client as MeiliClient, settings::Settings};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serenity::{
    client::Context,
    model::{application::component::ButtonStyle, channel::ReactionType},
    prelude::Mentionable,
};
use serenity::{
    model::{guild::Emoji, prelude::Message},
    utils::{read_image, MessageBuilder},
};
use std::{collections::HashMap, env, time::Duration};
use tokio::time::sleep;
use urlencoding::encode;

#[derive(Serialize, Deserialize, Debug)]
struct Thread {
    id: u64,
    guild_id: u64,
    channel_id: u64,
    title: String,
    history: String,
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

    let index_uid = "threads";

    let threads = {
        if let Ok(res) = mclient.get_index(index_uid).await {
            res
        } else {
            let task = mclient.create_index(index_uid, None).await.unwrap();
            let task = task
                .wait_for_completion(&mclient, None, None)
                .await
                .unwrap();
            let task = task.try_make_index(&mclient).unwrap();
            task.set_settings(&msettings).await.unwrap();
            task
        }
    };

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
                    "https://discord.com/channels/{}/{}",
                    ids.result.guild_id, ids.result.id
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

pub async fn responder(ctx: &Context, msg: &Message) -> Result<()> {
    if let Some(thread) = msg.channel(&ctx.http).await?.guild() {
        if let Some(parent_channel_id) = thread.parent_id {
            if [QUESTIONS_CHANNEL, SELFHOSTED_QUESTIONS_CHANNEL].contains(&parent_channel_id)
                && msg.id.as_u64() == thread.id.as_u64()
            {
                let user_mention = &msg.author.mention();
                let user_without_mention = &msg.author.name;

                thread
                .send_message(&ctx, |m| {
                    m.content( MessageBuilder::new().push_quote(format!("Hey {}! Thank you for raising this — please hang tight as someone from our community may help you out.", &user_without_mention)).build());

                    m.components(|c| {
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
                            ar.create_button(|button| {
                                button
                                    .style(ButtonStyle::Link)
                                    .label("Status")
                                    .emoji(ReactionType::Unicode("🧭".to_string()))
                                    .url("https://www.gitpodstatus.com/")
                            })
                        })
                    });

                    m

                })
                .await
                .unwrap();

                // questions_thread::responder(ctx).await;
                let thread_typing = thread.clone().start_typing(&ctx.http).unwrap();

                let relevant_links_external_sources = {
                    if parent_channel_id != SELFHOSTED_QUESTIONS_CHANNEL {
                        Vec::from(["https://www.gitpod.io/docs", "https://github.com/gitpod-io"])
                    } else {
                        Vec::from(["https://github.com/gitpod-io"])
                    }
                };

                let mut relevant_links = save_and_fetch_links(
                    &relevant_links_external_sources,
                    *thread.id.as_u64(),
                    *parent_channel_id.as_u64(),
                    *thread.guild_id.as_u64(),
                    thread.name.clone(),
                    (*msg.content).to_string(),
                )
                .await;

                if !&relevant_links.is_empty() {
                    let mut prefix_emojis: HashMap<&str, Emoji> = HashMap::new();
                    let emoji_sources: HashMap<&str, &str> = HashMap::from([
                        ("gitpod", "https://www.gitpod.io/images/media-kit/logo-mark.png"),
                        ("github", "https://cdn.discordapp.com/attachments/981191970024210462/981192908780736573/github-transparent.png"),
                        ("discord", "https://discord.com/assets/9f6f9cd156ce35e2d94c0e62e3eff462.png")
                    ]);
                    let guild = &msg.guild_id.unwrap();
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
                                let dw_path =
                                    env::current_dir().unwrap().join(format!("{source}.png"));
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
                m.content(format!("{} I also found some relevant links which might help to self-serve, please do check them out below 🙏:", &user_mention));
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

                                        a.create_button(|b|b.label(title.as_str().substring(0, 80)).custom_id(url.as_str().substring(0, 100)).style(ButtonStyle::Secondary).emoji(ReactionType::Custom {
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
                }

                // Take a pause
                sleep(Duration::from_secs(20)).await;

                let mut msg = MessageBuilder::new();
                msg.push_quote_line(format!(
                    "{} **{}**",
                    &user_mention, "You can share the following (if applies):"
                ));

                if parent_channel_id != SELFHOSTED_QUESTIONS_CHANNEL {
                    msg.push_line("• Contents of your `.gitpod.yml`")
                        .push_line("• Contents of your `.gitpod.Dockerfile`")
                        .push_line("• An example repository link");
                } else {
                    msg.push_line("• Contents of your `config.yml`")
                    .push("• Result of:```bash\nkubectl get pods -n <namespace>```")
                    .push_line("• If you have resources that are set up strangely, please run `kubectl describe` on the resource");
                }

                thread
                    .send_message(&ctx.http, |m| m.content(msg.build()))
                    .await
                    .unwrap();

                thread_typing.stop().unwrap();
            }
        }
    };

    Ok(())
}

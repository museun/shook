use std::{collections::BTreeSet, time::SystemTime};

use anyhow::Context;
use shook_core::{help::Descriptions, prelude::*, FormatTime};
use shook_local::LocalPort;
use tokio::time::Instant;

pub struct Builtin(Instant);

impl Builtin {
    pub async fn bind(state: GlobalState) -> anyhow::Result<SharedCallable> {
        let registry = state.get().await;
        Ok(Binding::create(&registry, Self(Instant::now()))
            .bind(Self::theme)
            .bind(Self::font)
            .bind(Self::uptime)
            .bind(Self::bot_uptime)
            .bind(Self::time)
            .bind(Self::hello)
            .bind(Self::help)
            .bind(Self::version)
            .bind(Self::local_port)
            .listen(Self::say_hello)
            .into_callable())
    }

    async fn local_port(self: Arc<Self>, msg: Message) -> impl Render {
        msg.require_broadcaster()?;
        Ok(msg.state().get::<LocalPort>().await.to_string())
    }

    async fn version(self: Arc<Self>, _: Message) -> impl Render {
        Simple {
            twitch: format!(
                "{} on branch '{}' (built on {})",
                crate::GIT_REVISION,
                crate::GIT_BRANCH,
                crate::BUILD_TIME
            ),
            discord: format!(
                "`{}` on branch `{}` (built on `{}`)",
                crate::GIT_REVISION,
                crate::GIT_BRANCH,
                crate::BUILD_TIME
            ),
        }
    }

    async fn help(self: Arc<Self>, msg: Message) -> impl Render {
        return match msg.args().get("command") {
            Some(cmd) if !cmd.starts_with('!') => {
                anyhow::bail!("you must prefix commands with !")
            }

            // TODO list aliases
            Some(cmd) => {
                let registry = msg.state().get::<Registry>().await;
                match registry.find_command(cmd) {
                    Some(desc) => Ok(Simple {
                        twitch: format!(
                            "{usage} | {desc}",
                            usage = desc.usage(),
                            desc = desc.description()
                        ),
                        discord: format!(
                            "`{usage}` | {desc}",
                            usage = desc.usage(),
                            desc = desc.description()
                        ),
                    }
                    .boxed()),
                    None => anyhow::bail!("cannot find '{cmd}'"),
                }
            }
            None => {
                let registry = msg.state().get::<Registry>().await;
                let f = format_help_twitch(
                    &registry
                        .get_all_descriptions()
                        .flat_map(Descriptions::command_names)
                        .collect(),
                );
                Ok(Simple {
                    twitch: f.clone(),
                    discord: f,
                }
                .boxed())
            }
        };

        fn format_help_twitch(desc: &BTreeSet<&str>) -> Vec<Response> {
            const MAX: usize = 10;
            let (mut left, right) = desc.iter().enumerate().fold(
                (Response::builder(), String::new()),
                |(mut left, mut right), (i, c)| {
                    if i != 0 && i % MAX == 0 {
                        left = left.say(std::mem::take(&mut right))
                    }
                    if !right.is_empty() {
                        right.push(' ')
                    }
                    right.push_str(c);
                    (left, right)
                },
            );

            if !right.trim().is_empty() {
                left = left.say(right)
            }

            left.finish()
        }
    }

    async fn hello(self: Arc<Self>, msg: Message) -> impl Render {
        format!("hello, {}!", msg.sender_name())
    }

    async fn say_hello(self: Arc<Self>, msg: Message) -> impl Render {
        let data = msg.data().trim_end_matches(['!', '?', '.']);
        if matches!(data, s if !s.eq_ignore_ascii_case("hello")) {
            return None;
        }
        Some(format!("hello, {}.", msg.sender_name()))
    }

    async fn time(self: Arc<Self>, _: Message) -> impl Render {
        let f = time::format_description::parse("[hour]:[minute]:[second]")?;
        let now = time::OffsetDateTime::now_local()?.format(&f)?;

        Ok(Simple {
            twitch: format!("current time: {now}"),
            discord: format!("current time: `{now}`"),
        })
    }

    async fn bot_uptime(self: Arc<Self>, _: Message) -> impl Render {
        let uptime = self.0.elapsed().as_readable_time();
        Simple {
            twitch: format!("I've been running for: {uptime}"),
            discord: format!("I've been running for: `{uptime}`"),
        }
    }

    async fn uptime(self: Arc<Self>, msg: Message) -> impl Render {
        let channel = match msg.args().get("channel") {
            Some(channel) => channel.to_string(),
            None => msg.streamer_name().await,
        };

        let client = msg.state().get::<shook_helix::HelixClient>().await;
        if let [stream] = &*client.get_streams([&channel]).await? {
            let uptime = (SystemTime::now() - stream.started_at).as_readable_time();
            return Ok(Simple {
                twitch: format!("'{channel}' has been live for: {uptime}"),
                discord: format!("<https://twitch.tv/{channel}> has been live for: `{uptime}`"),
            });
        }

        anyhow::bail!("I don't know")
    }

    async fn theme(self: Arc<Self>, _: Message) -> impl Render {
        let current = what_theme::get_current_theme()?;
        let settings = what_theme::VsCodeSettings::new()?;

        let theme = settings
            .find_theme(&current)
            .with_context(|| "I can't figure that out")?;

        let url = theme.url();
        let variant = theme.variant();
        Ok(Simple {
            twitch: format!("'{variant}' from {url}"),
            discord: format!("`{variant}` from <{url}>"),
        })
    }

    async fn font(self: Arc<Self>, _: Message) -> impl Render {
        let fonts = what_theme::get_current_fonts()?;
        let (editor, terminal) = (fonts.editor(), fonts.terminal());
        Ok(Simple {
            twitch: format!("terminal is using: '{editor}' and editor is using '{terminal}'"),
            discord: format!("terminal is using: `{editor}` and editor is using `{terminal}`"),
        })
    }
}

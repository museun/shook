use std::time::SystemTime;

use anyhow::Context;
use shook::{prelude::*, FormatTime};
use tokio::time::Instant;

pub struct Builtin(Instant);

pub async fn bind(state: GlobalState) -> anyhow::Result<SharedCallable> {
    Builtin::bind(state).await
}

impl Builtin {
    async fn bind(_: GlobalState) -> anyhow::Result<SharedCallable> {
        let theme_cmd = cmd("!theme").help("tries to look up the current vscode theme");
        let uptime_cmd = cmd("!uptime")
            .help("retrieves a stream's current uptime")
            .usage("<channel?>")?;
        let bot_uptime_cmd = cmd("!bot-uptime").help("retrieves the bot's current uptime");
        let time_cmd = cmd("!time").help("retrieves the stream's current time");
        let hello_cmd = cmd("!hello").help("gives a greeting");

        Ok(Binding::create(Self(Instant::now()))
            .bind(theme_cmd, Self::theme)
            .bind(uptime_cmd, Self::uptime)
            .bind(bot_uptime_cmd, Self::bot_uptime)
            .bind(time_cmd, Self::time)
            .bind(hello_cmd, Self::hello)
            .listen(Self::say_hello)
            .into_callable())
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

        let client = msg.state().get::<shook::helix::HelixClient>().await;
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
}

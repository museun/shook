use shook::{prelude::*, FormatTime};
use tokio::time::Instant;

pub struct Builtin(Instant);

impl Builtin {
    pub async fn bind(_: GlobalState) -> anyhow::Result<SharedCallable> {
        let theme_cmd = cmd("!theme").help("tries to look up the current vscode theme");
        let uptime_cmd = cmd("!uptime").help("retrieves the stream's current uptime");
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
        Ok(format!("current time: {now}"))
    }

    async fn bot_uptime(self: Arc<Self>, _: Message) -> impl Render {
        let uptime = self.0.elapsed().as_readable_time();
        format!("I've been running for: {uptime}")
    }

    async fn uptime(self: Arc<Self>, _: Message) -> impl Render {}

    async fn theme(self: Arc<Self>, _: Message) -> impl Render {
        let current = what_theme::get_current_theme()?;
        let settings = what_theme::VsCodeSettings::new()?;

        return match settings.find_theme(&current) {
            Some(theme) => Ok(theme.to_string()),
            None => anyhow::bail!("I can't figure that out"),
        };
    }
}

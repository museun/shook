use std::time::Duration;

use fastrand_ext::IterExt;
use once_cell::sync::Lazy;
use regex::Regex;
use shook_config::Secret;
use shook_core::prelude::*;
use shook_helix::EmoteMap;
use tokio::{sync::Mutex, time::Instant};

pub async fn bind(state: GlobalState) -> anyhow::Result<SharedCallable> {
    AnotherViewer::bind(state).await
}

struct AnotherViewer {
    last: Mutex<Instant>,
    emote_map: EmoteMap,
    client: reqwest::Client,
    key: Secret<String>,
    remote: String,
}

impl AnotherViewer {
    async fn bind(state: GlobalState) -> anyhow::Result<SharedCallable> {
        let config: crate::config::AnotherViewer = state.get_owned().await;

        let this = Self {
            last: Mutex::new(Instant::now()),
            emote_map: state.get_owned().await,
            client: reqwest::Client::new(),
            key: config.bearer_token,
            remote: config.remote,
        };

        let reg = state.get().await;
        Ok(Binding::create(&reg, this)
            .bind(Self::speak)
            .listen(Self::listen)
            .into_callable())
    }

    async fn speak(self: Arc<Self>, msg: Message) -> impl Render {
        let ctx = msg.args().get("context");
        self.generate(ctx).await.map(Response::reply)
    }

    async fn listen(self: Arc<Self>, msg: Message) -> impl Render {
        if msg.data().starts_with('!') {
            return None;
        }

        // TODO not in this task
        let _ = self.train(msg.data()).await;
        let s = msg.data().split_ascii_whitespace().collect::<Vec<_>>();

        if let Some(msg) = self.try_mention(msg.sender_name(), &s).await {
            return Some(msg.boxed());
        }

        if let Some(msg) = self.try_kappa(&s).await {
            return Some(msg.boxed());
        }

        if !self.check_timeout().await {
            return None;
        }
        self.generate(None).await.map(|r| r.boxed())
    }

    async fn try_mention<'a>(&self, sender: &str, ctx: &'a [&'a str]) -> Option<impl Render> {
        if Self::contains_bot_name(ctx) {
            return self.generate(Some(sender)).await.map(Response::reply);
        }
        None
    }

    async fn try_kappa<'a>(&self, ctx: &'a [&'a str]) -> Option<impl Render> {
        let kappa = ctx
            .iter()
            .filter(|c| self.emote_map.has(c))
            .choose(&fastrand::Rng::new())?;
        self.generate(Some(kappa)).await
    }

    async fn check_timeout(&self) -> bool {
        const COOLDOWN: Duration = Duration::from_secs(60);
        let mut last = self.last.lock().await;
        if last.elapsed() >= COOLDOWN {
            *last = Instant::now();
            return true;
        }
        false
    }

    async fn train(&self, data: &str) -> Option<()> {
        static BANNED: Lazy<Regex> =
            Lazy::new(|| Regex::new(r#"((!|@).+?\b)|(\bshaken(_bot)?\b)"#).unwrap());
        let next = BANNED.replace_all(data, "");

        use shook_core::IterExt as _;
        let data = next
            .split_ascii_whitespace()
            .filter(|c| url::Url::parse(c).is_err())
            .join_with(' ');

        #[derive(serde::Serialize)]
        struct Req {
            data: String,
        }

        self.client
            .post(format!("{}/train", &self.remote))
            .bearer_auth(&*self.key)
            .json(&Req { data })
            .send()
            .await
            .ok()
            .map(drop)
    }

    async fn generate(&self, context: Option<&str>) -> Option<String> {
        #[derive(serde::Deserialize)]
        struct Resp {
            data: String,
        }

        #[derive(serde::Serialize)]
        struct Req {
            min: usize,
            max: usize,
            context: Option<String>,
        }

        let req = Req {
            min: 3,
            max: 50,
            context: context.map(ToString::to_string),
        };

        self.client
            .get(format!("{}/generate", &self.remote))
            .json(&req)
            .timeout(Duration::from_secs(3))
            .send()
            .await
            .ok()?
            .json::<Resp>()
            .await
            .ok()
            .map(|c| c.data)
    }

    fn contains_bot_name(ctx: &[&str]) -> bool {
        fn match_name(s: &str) -> bool {
            const HEAD: &[char] = &['(', '[', '{', '@', '\'', '"'];
            const TAIL: &[char] = &['"', '\'', ',', '.', '?', ':', ';', '!', '}', ']', ')'];
            matches!(
                s.trim_start_matches(HEAD).trim_end_matches(TAIL),
                "shaken" | "shaken_bot"
            )
        }

        ctx.iter().copied().any(match_name)
    }
}

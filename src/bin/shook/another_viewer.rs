use std::time::Duration;

use fastrand_ext::IterExt;
use once_cell::sync::Lazy;
use regex::Regex;
use shook::{helix::EmoteMap, prelude::*};
use tokio::{sync::Mutex, time::Instant};

pub async fn bind(state: GlobalState) -> anyhow::Result<SharedCallable> {
    AnotherViewer::bind(state).await
}

struct AnotherViewer {
    last: Mutex<Instant>,
    emote_map: EmoteMap,
    client: reqwest::Client,
}

impl AnotherViewer {
    async fn bind(state: GlobalState) -> anyhow::Result<SharedCallable> {
        let this = Self {
            last: Mutex::new(Instant::now()),
            emote_map: state.get::<EmoteMap>().await.clone(),
            client: reqwest::Client::new(),
        };

        let reg = state.get().await;
        Ok(Binding::create(&reg, this)
            .bind("another_viewer::speak", Self::speak)
            .listen(Self::listen)
            .into_callable())
    }

    async fn speak(self: Arc<Self>, msg: Message) -> impl Render {
        let ctx = msg.args().get("context");
        self.generate(ctx.as_deref()).await
    }

    async fn listen(self: Arc<Self>, msg: Message) -> impl Render {
        if msg.data().starts_with('!') {
            return None;
        }

        // TODO not in this task
        let _ = self.train(msg.data()).await;

        let s = msg.data().split_ascii_whitespace().collect::<Vec<_>>();
        if let Some(msg) = self.try_kappa(&s).await {
            return Some(msg);
        }

        if let Some(msg) = self.try_mention(msg.sender_name(), &s).await {
            return Some(msg);
        }

        if !self.check_timeout().await {
            return None;
        }
        self.generate(None).await
    }

    async fn try_mention<'a>(&self, sender: &str, ctx: &'a [&'a str]) -> Option<String> {
        if Self::contains_bot_name(ctx) {
            return self.generate(Some(sender)).await;
        }
        None
    }

    async fn try_kappa<'a>(&self, ctx: &'a [&'a str]) -> Option<String> {
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
        const BRAIN: &str = "http://localhost:50000/museun/train";

        static BANNED: Lazy<Regex> =
            Lazy::new(|| Regex::new(r#"((!|@).+?\b)|(\bshaken(_bot)?\b)"#).unwrap());

        let next = BANNED.replace_all(data, "");

        use shook::IterExt as _;
        let data = next
            .split_ascii_whitespace()
            .filter(|c| url::Url::parse(c).is_err())
            .join_with(' ');

        #[derive(serde::Serialize)]
        struct Req {
            data: String,
        }

        self.client
            .post(BRAIN)
            .json(&Req { data })
            .send()
            .await
            .ok()
            .map(drop)
    }

    async fn generate(&self, context: Option<&str>) -> Option<String> {
        const BRAIN: &str = "http://localhost:50000/museun/generate";
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

        // TODO min/max
        let resp: Resp = self
            .client
            .get(BRAIN)
            .json(&Req {
                min: 3,
                max: 50,
                context: context.map(ToString::to_string),
            })
            .timeout(Duration::from_secs(3))
            .send()
            .await
            .ok()?
            .json()
            .await
            .ok()?;

        Some(resp.data)
    }

    fn contains_bot_name<'a>(ctx: &'a [&'a str]) -> bool {
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

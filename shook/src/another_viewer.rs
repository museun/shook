use std::time::Duration;

use fastrand_ext::IterExt as _;
use once_cell::sync::Lazy;
use regex::Regex;
use shook_config::Secret;
use shook_core::{prelude::*, IterExt as _};
use shook_helix::EmoteMap;
use tokio::{sync::Mutex, time::Instant};

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct BanPatterns {
    pub list: Vec<String>,
}

struct Patterns {
    pub patterns: Vec<Regex>,
}

pub struct AnotherViewer {
    last: Mutex<Instant>,
    emote_map: EmoteMap,
    client: reqwest::Client,
    key: Secret<String>,
    remote: String,
    patterns: Patterns,
}

impl AnotherViewer {
    pub async fn bind(state: GlobalState) -> anyhow::Result<SharedCallable> {
        let config: crate::config::AnotherViewer = state.get_owned().await;
        let ban_patterns: &BanPatterns = &*state.get().await;

        let patterns = match ban_patterns
            .list
            .iter()
            .map(|s| Regex::new(s).map_err(Into::into))
            .collect::<anyhow::Result<Vec<_>>>()
        {
            Ok(patterns) => patterns,
            Err(err) => {
                log::error!("invalid pattern. try again");
                return Err(err);
            }
        };

        let this = Self {
            last: Mutex::new(Instant::now()),
            emote_map: state.get_owned().await,
            client: reqwest::Client::new(),
            key: config.bearer_token,
            remote: config.remote,
            patterns: Patterns { patterns },
        };

        let reg = state.get().await;
        Ok(Binding::create(&reg, this)
            .bind(Self::speak)
            .bind(Self::banned)
            .listen(Self::listen)
            .into_callable())
    }

    async fn speak(self: Arc<Self>, msg: Message) -> impl Render {
        let ctx = msg.args().get("context");
        self.generate(ctx).await.map(Response::reply)
    }

    async fn banned(self: Arc<Self>, msg: Message) -> impl Render {
        msg.require_elevation()?;

        Ok(self
            .patterns
            .patterns
            .iter()
            .map(|s| s.as_str())
            .fold(Response::builder(), |r, s| r.say(s))
            .finish())
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
        #[derive(serde::Serialize)]
        struct Req {
            data: String,
        }

        let data = self.process_data(data);

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

        #[derive(Debug, serde::Serialize)]
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
            .map(|c| self.process_data(&c.data))
    }

    fn process_data(&self, data: &str) -> String {
        let data = &mut String::from(data);
        [
            Self::filter_mentions,
            Self::filter_annoying_patterns,
            Self::filter_urls,
            // this has to be last
            Self::collapse_whitespace,
        ]
        .into_iter()
        .for_each(|f| *data = f(self, &*data));
        data.to_string()
    }

    fn filter_annoying_patterns(&self, data: &str) -> String {
        self.patterns
            .patterns
            .iter()
            .fold(String::with_capacity(data.len()), |mut text, re| {
                text.push_str(&*re.replace_all(data, ""));
                text
            })
    }

    fn filter_mentions(&self, input: &str) -> String {
        static PATTERN: Lazy<Regex> =
            Lazy::new(|| Regex::new(r#"((!|@).+?\b)|(\bshaken(_bot)?\b)"#).unwrap());
        PATTERN.replace_all(input, "").to_string()
    }

    fn filter_urls(&self, input: &str) -> String {
        input
            .split_ascii_whitespace()
            .filter(|c| url::Url::parse(c).is_err())
            .join_with(' ')
    }

    fn collapse_whitespace(&self, input: &str) -> String {
        static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r#"\s{2,}"#).unwrap());
        PATTERN.replace_all(input, " ").to_string()
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

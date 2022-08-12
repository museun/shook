use persist::{
    json::{Json, JsonPretty},
    tokio::PersistExt,
};
use std::collections::HashMap;

use shook_core::{help::Registry, prelude::*};
use tokio::sync::Mutex;

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, PartialOrd, Eq, Ord)]
struct Command {
    name: String,
    body: String,
    author: String,
    uses: usize,
}

impl Command {
    fn new(name: impl Into<String>, body: impl Into<String>, author: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            body: body.into(),
            author: author.into(),
            uses: 0,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default, Debug)]
struct UserDefinedState {
    map: HashMap<String, Command>,
    aliases: Vec<(String, String)>,
}

impl UserDefinedState {
    pub fn insert(&mut self, command: Command) -> bool {
        if self.map.contains_key(&command.name)
            || self.aliases.iter().any(|(c, _)| c == &command.name)
        {
            return false;
        }

        self.map.insert(command.name.clone(), command);
        true
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.aliases.retain(|(k, v)| k != name && v != name);
        self.map.remove(name).is_some()
    }

    pub fn update(&mut self, name: &str, update: impl Fn(&mut Command)) -> bool {
        if let Some(name) = self.find_name(name).map(ToString::to_string) {
            return self.map.get_mut(&name).map(update).is_some();
        }
        false
    }

    pub fn alias(&mut self, from: &str, to: &str) -> bool {
        // TODO disable cyclic aliasing
        if !self.has(from) || self.has(to) {
            return false;
        }

        if self.aliases.iter().any(|(k, v)| (k == from) ^ (v == to)) {
            return true;
        }

        self.aliases.push((from.to_string(), to.to_string()));
        true
    }

    pub fn get_by_name(&self, name: &str) -> Option<&Command> {
        self.map.get(name).or_else(|| {
            for (k, v) in &self.aliases {
                if v == name {
                    return self.map.get(k);
                }
            }
            None
        })
    }

    pub fn find_name<'a>(&'a self, name: &'a str) -> Option<&'a str> {
        if self.map.contains_key(name) {
            return Some(name);
        }
        for (k, v) in &self.aliases {
            if v == name {
                return Some(k);
            }
        }

        None
    }

    pub fn get_all(&self) -> impl Iterator<Item = &Command> {
        self.map.values()
    }

    pub fn has(&self, name: &str) -> bool {
        self.get_by_name(name).is_some()
    }
}

pub async fn bind(state: GlobalState) -> anyhow::Result<SharedCallable> {
    UserDefined::bind(state).await
}

pub struct UserDefined {
    state: Mutex<UserDefinedState>,
}

impl UserDefined {
    async fn bind(state: GlobalState) -> anyhow::Result<SharedCallable> {
        let uds = UserDefinedState::load_from_file::<Json>("user_defined")
            .await
            .unwrap_or_default();

        let this = Self {
            state: Mutex::new(uds),
        };

        let reg = state.get::<Registry>().await;
        Ok(Binding::create(&reg, this)
            .bind(Self::add)
            .bind(Self::update)
            .bind(Self::remove)
            .bind(Self::alias)
            .bind(Self::commands)
            .listen(Self::lookup)
            .into_callable())
    }

    async fn add(self: Arc<Self>, msg: Message) -> impl Render {
        msg.require_elevation()?;

        let name = Self::validate_command(&msg.args()["name"])?;
        let body = &msg.args()["body"];
        anyhow::ensure!(!body.is_empty(), "the command body cannot be empty");

        let mut state = self.state.lock().await;
        if !state.insert(Command::new(name, body, msg.sender_name())) {
            return Ok(Simple {
                twitch: format!("{name} already exists"),
                discord: format!("`{name}` already exists"),
            });
        }

        Self::sync(&state).await?;

        Ok(Simple {
            twitch: format!("created {name} -> {body}"),
            discord: format!("created `{name}` -> `{body}`"),
        })
    }

    async fn update(self: Arc<Self>, msg: Message) -> impl Render {
        msg.require_elevation()?;

        let name = Self::validate_command(&msg.args()["name"])?;
        let body = &msg.args()["body"];

        anyhow::ensure!(!body.is_empty(), "the command body cannot be empty");

        let mut state = self.state.lock().await;
        if !state.update(name, |cmd| cmd.body = body.to_string()) {
            return Ok(Simple {
                twitch: format!("{name} doesn't exists"),
                discord: format!("`{name}` doesn't exists"),
            });
        }

        Self::sync(&state).await?;

        Ok(Simple {
            twitch: format!("updated {name} -> {body}"),
            discord: format!("updated `{name}` -> `{body}`"),
        })
    }

    async fn remove(self: Arc<Self>, msg: Message) -> impl Render {
        msg.require_elevation()?;

        let name = Self::validate_command(&msg.args()["name"])?;

        let mut state = self.state.lock().await;
        if !state.remove(name) {
            return Ok(Simple {
                twitch: format!("{name} wasn't found"),
                discord: format!("`{name}` wasn't found"),
            });
        }

        Self::sync(&state).await?;

        Ok(Simple {
            twitch: format!("removed: {name}"),
            discord: format!("removed: `{name}`"),
        })
    }

    async fn alias(self: Arc<Self>, msg: Message) -> impl Render {
        msg.require_elevation()?;

        let from = Self::validate_command(&msg.args()["from"])?;
        let to = Self::validate_command(&msg.args()["to"])?;

        let mut state = self.state.lock().await;
        if !state.has(from) {
            return Ok(Simple {
                twitch: format!("{from} was not found"),
                discord: format!("`{from}` was not found"),
            });
        }

        if !state.alias(from, to) {
            return Ok(Simple {
                twitch: format!("{to} already exists"),
                discord: format!("`{to}` already exists"),
            });
        }

        Self::sync(&state).await?;

        Ok(Simple {
            twitch: format!("aliased {from} to {to}"),
            discord: format!("aliased `{from}` to `{to}`"),
        })
    }

    async fn commands(self: Arc<Self>, _: Message) -> impl Render {
        let state = self.state.lock().await;
        let (mut resp, line) = state.get_all().map(|c| &*c.name).enumerate().fold(
            (Response::builder(), String::new()),
            |(mut resp, mut out), (i, cmd)| {
                if i > 0 && i % 20 == 0 {
                    resp = resp.say(std::mem::take(&mut out))
                }
                if !out.is_empty() {
                    out.push(' ')
                }
                out.push_str(cmd);
                (resp, out)
            },
        );

        if !line.is_empty() {
            resp = resp.say(line)
        }
        resp
    }

    async fn lookup(self: Arc<Self>, msg: Message) -> impl Render {
        let cmd = msg.data().split_ascii_whitespace().next()?;
        let mut state = self.state.lock().await;
        if !state.has(cmd) {
            return None;
        }

        state.update(cmd, |cmd| cmd.uses += 1);
        let cmd = state.get_by_name(cmd).expect("cmd should exist");
        Some(cmd.body.clone())
    }

    async fn sync(state: &UserDefinedState) -> anyhow::Result<()> {
        Ok(state.save_to_file::<JsonPretty>("user_defined").await?)
    }

    fn validate_command(name: &str) -> anyhow::Result<&str> {
        anyhow::ensure!(name.starts_with('!'), "you must prefix commands with !");
        anyhow::ensure!(name.len() > 1, "the command name cannot be empty");
        Ok(name)
    }
}

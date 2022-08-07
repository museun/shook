use std::{borrow::Cow, collections::HashMap};

use crate::{
    persist::{Json, Lexpr, Ron, Toml},
    prelude::Command,
};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Registry {
    #[serde(flatten)]
    map: HashMap<String, Descriptions>,
}

impl Registry {
    pub fn descriptions_for(&self, namespace: &str) -> Option<&Descriptions> {
        self.map.get(namespace)
    }

    pub fn get_from_id(&self, id: &str) -> Option<anyhow::Result<Command>> {
        let (head, tail) = id.split_once("::")?;
        let tail = tail
            .starts_with('!')
            .then_some(Cow::from(tail))
            .unwrap_or_else(|| Cow::from(format!("!{tail}")));
        self.get_command(head, &tail)
    }

    pub fn get_command(&self, namespace: &str, cmd: &str) -> Option<anyhow::Result<Command>> {
        self.descriptions_for(namespace)?
            .get(cmd)
            .map(|c| c.parse_command())
    }
}

#[tokio::test]
async fn foo() {
    use crate::persist::{PersistExt, Yaml};
    use crate::prelude::cmd;

    let reg = {
        let mut reg = Registry::default();
        reg.map.insert(String::from("builtin"), {
            let theme_cmd = cmd("!theme").help("tries to look up the current vscode theme");
            let uptime_cmd = cmd("!uptime")
                .help("retrieves a stream's current uptime")
                .usage("<channel?>")
                .unwrap();
            let bot_uptime_cmd = cmd("!bot-uptime").help("retrieves the bot's current uptime");
            let time_cmd = cmd("!time").help("retrieves the stream's current time");
            let hello_cmd = cmd("!hello").alias("!greet").help("gives a greeting");

            [theme_cmd, uptime_cmd, bot_uptime_cmd, time_cmd, hello_cmd]
                .into_iter()
                .fold(Descriptions::default(), |desc, cmd| desc.with(cmd.into()))
        });

        reg.map.insert(String::from("crates"), {
            Descriptions::default().with(
                cmd("!crate")
                    .alias("!crates")
                    .alias("!lookup")
                    .help("look up a Rust crate")
                    .usage("<name>")
                    .unwrap()
                    .into(),
            )
        });

        reg.map.insert(String::from("spotify"), {
            Descriptions::default()
                .with(
                    cmd("!song")
                        .alias("!current")
                        .help("gets the currently playing song from spotify")
                        .into(),
                )
                .with(
                    cmd("!previous")
                        .help("gets the previously played song from spotify")
                        .into(),
                )
        });

        reg
    };

    reg.save_to_file::<Json>(&"default_help").await.unwrap();
    reg.save_to_file::<Toml>(&"default_help").await.unwrap();
    reg.save_to_file::<Yaml>(&"default_help").await.unwrap();
    reg.save_to_file::<Lexpr>(&"default_help").await.unwrap();
    reg.save_to_file::<Ron>(&"default_help").await.unwrap();
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct Descriptions {
    descriptions: Vec<Description>,
}

impl Descriptions {
    pub fn get(&self, key: &str) -> Option<&Description> {
        self.descriptions.iter().find(|c| c.matches_command(key))
    }

    pub fn with(mut self, desc: Description) -> Self {
        self.add(desc);
        self
    }

    pub fn add(&mut self, desc: Description) {
        self.descriptions.push(desc)
    }

    pub fn command_names(&self) -> impl Iterator<Item = &str> {
        self.descriptions.iter().flat_map(|c| {
            [&*c.command]
                .into_iter()
                .chain(c.aliases.iter().map(|c| &**c))
        })
    }

    pub fn description_for(&self, name: &str) -> Option<&str> {
        self.get(name).map(|d| d.description())
    }

    pub fn usage_for(&self, name: &str) -> Option<Cow<'_, str>> {
        self.get(name).map(|d| d.usage())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Description {
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<String>,
    pub description: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
}

impl Description {
    pub fn parse_command(&self) -> anyhow::Result<Command> {
        let mut cmd = self.aliases.iter().fold(
            Command::new(&self.command).help(&self.description),
            |cmd, a| cmd.alias(a),
        );
        if let Some(usage) = &self.usage {
            cmd = cmd.usage(usage)?;
        }
        Ok(cmd)
    }

    pub fn matches_command(&self, input: &str) -> bool {
        [&*self.command]
            .into_iter()
            .chain(self.aliases.iter().map(|s| &**s))
            .any(|c| c == input)
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn usage(&self) -> Cow<'_, str> {
        let cmd = &self.command;
        match &self.usage {
            Some(usage) => Cow::from(format!("{cmd} {usage}")),
            None => Cow::from(cmd),
        }
    }
}

impl<'a> From<&'a Command> for Description {
    fn from(cmd: &'a Command) -> Self {
        Self {
            command: cmd.command.to_string(),
            aliases: cmd.aliases.iter().map(|s| s.to_string()).collect(),
            description: cmd
                .description
                .as_ref()
                .map(|s| s.to_string())
                .or_else(|| cmd.example.as_ref().map(|c| c.usage.to_string()))
                .unwrap_or_else(|| cmd.command.to_string()),
            usage: cmd.example.as_ref().map(|c| c.usage.to_string()),
        }
    }
}

impl From<Command> for Description {
    fn from(cmd: Command) -> Self {
        (&cmd).into()
    }
}

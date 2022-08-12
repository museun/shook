use std::{borrow::Cow, collections::HashMap};

use crate::prelude::Command;

#[macro_export]
macro_rules! cmd {
    ($namespace:tt :: $command:tt) => {
        concat!(stringify!($namespace), "::", stringify!($command))
    };
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Registry {
    #[serde(flatten)]
    map: HashMap<String, Descriptions>,
}

impl Registry {
    pub fn descriptions_for(&self, namespace: &str) -> Option<&Descriptions> {
        self.map.get(namespace)
    }

    pub fn fetch(&self, id: &str) -> Command {
        let (head, tail) = id.split_once("::").expect("invalid command id");
        let tail = tail
            .starts_with('!')
            .then_some(Cow::from(tail))
            .unwrap_or_else(|| Cow::from(format!("!{tail}")));
        self.get_command(head, &tail)
            .unwrap_or_else(|| panic!("'{id}' was invalid"))
            .unwrap()
    }

    pub fn get_command(&self, namespace: &str, cmd: &str) -> Option<anyhow::Result<Command>> {
        self.descriptions_for(namespace)?
            .get(cmd)
            .map(Description::parse_command)
    }

    pub fn get_all_descriptions(&self) -> impl Iterator<Item = &Descriptions> {
        self.map.values()
    }

    pub fn find_command(&self, cmd: &str) -> Option<&Description> {
        self.map.values().find_map(|desc| desc.get(cmd))
    }

    pub fn add(&mut self, namespace: &str, desc: impl Into<Description>) {
        self.map
            .entry(namespace.to_string())
            .or_default()
            .add(desc.into());
    }
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
        self.descriptions.iter().flat_map(Description::commands)
    }

    pub fn description_for(&self, name: &str) -> Option<&str> {
        self.get(name).map(Description::description)
    }

    pub fn usage_for(&self, name: &str) -> Option<Cow<'_, str>> {
        self.get(name).map(Description::usage)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Description {
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub usage: Option<String>,
    pub description: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
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

    pub fn commands(&self) -> impl Iterator<Item = &str> {
        [&*self.command]
            .into_iter()
            .chain(self.aliases.iter().map(|s| &**s))
    }

    pub fn matches_command(&self, input: &str) -> bool {
        self.commands().any(|c| c == input)
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
            aliases: cmd.aliases.iter().map(<_>::to_string).collect(),
            description: cmd
                .description
                .as_ref()
                .map(<_>::to_string)
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

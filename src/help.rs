use crate::prelude::Command;

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct Descriptions {
    descriptions: Vec<Description>,
}

impl Descriptions {
    pub fn get(&self, key: &str) -> Option<&Description> {
        self.descriptions.iter().find(|c| {
            [&*c.command]
                .into_iter()
                .chain(c.aliases.iter().map(|c| &**c))
                .fold(false, |ok, s| ok ^ (key == s))
        })
    }

    pub fn add(&mut self, desc: Description) {
        self.descriptions.push(desc)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Description {
    pub command: String,
    pub aliases: Vec<String>,
    pub description: String,
    pub usage: Option<String>,
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

use std::sync::Arc;

use crate::args::ExampleArgs;

#[derive(Clone, Debug)]
pub struct Command {
    pub command: Arc<str>,
    pub aliases: Vec<Arc<str>>,
    pub description: Option<Arc<str>>,
    pub example: Option<Arc<ExampleArgs>>,
}

impl Command {
    pub fn new(command: &str) -> Self {
        Self {
            command: command.into(),
            aliases: Vec::new(),
            description: None,
            example: None,
        }
    }

    pub fn alias(mut self, alias: &str) -> Self {
        self.aliases.push(Arc::from(alias));
        self
    }

    pub fn usage(mut self, usage: &str) -> anyhow::Result<Self> {
        let example = ExampleArgs::parse(usage).map(Arc::new)?;
        self.example.get_or_insert(example);
        Ok(self)
    }

    pub fn help(mut self, help: &str) -> Self {
        self.description.get_or_insert_with(|| Arc::from(help));
        self
    }
}

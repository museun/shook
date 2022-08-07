use std::sync::Arc;

use super::{Privmsg, Tags};

pub struct Message {
    pub(crate) sender: Arc<str>,
    target: Arc<str>,
    pub(crate) data: Arc<str>,
    tags: Arc<Tags>,
}

impl Message {
    pub fn from_pm(pm: Privmsg) -> Self {
        Self {
            sender: pm.user,
            target: pm.target,
            data: pm.data,
            tags: Arc::new(pm.tags),
        }
    }

    pub fn channel(&self) -> &str {
        &self.target
    }

    pub fn tags(&self) -> &Tags {
        &self.tags
    }
}

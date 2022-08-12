use std::sync::Arc;

use shook_core::message::MessageType;

use super::{Privmsg, Tags};

#[derive(Debug)]
pub struct Message {
    sender: Arc<str>,
    target: Arc<str>,
    data: Arc<str>,
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

    fn badge_iter(&self) -> impl Iterator<Item = (&str, &str)> + '_ {
        self.tags
            .get("badges")
            .into_iter()
            .flat_map(|s| s.split(','))
            .flat_map(|s| s.split_once('/'))
    }
}

impl MessageType for Message {
    fn data(&self) -> &str {
        &self.data
    }

    fn sender_name(&self) -> &str {
        &self.sender
    }

    fn source(&self) -> &str {
        &self.target
    }

    fn is_from_admin(&self) -> bool {
        self.badge_iter()
            .any(|(key, val)| key == "broadcaster" && val == "1")
    }

    fn is_from_moderator(&self) -> bool {
        self.badge_iter()
            .any(|(key, val)| key == "moderator" && val == "1")
    }
}

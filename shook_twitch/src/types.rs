use std::sync::Arc;

use super::Tags;

pub struct Privmsg {
    pub tags: Tags,
    pub user: Arc<str>,
    pub target: Arc<str>,
    pub data: Arc<str>,
}

#[derive(Debug, Clone)]
pub struct Identity {
    pub name: Box<str>,
    pub user_id: u64,
}

#[derive(Copy, Clone)]
pub struct Registration<'a> {
    pub name: &'a str,
    pub pass: &'a str,
}

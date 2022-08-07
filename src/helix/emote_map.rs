use std::collections::{HashMap, HashSet};

#[derive(Clone, Default)]
pub struct EmoteMap {
    name_to_id: HashMap<Box<str>, Box<str>>,
    id_to_name: HashMap<Box<str>, Box<str>>,
    names: HashSet<Box<str>>,
}

impl EmoteMap {
    pub fn with_emotes<'k, 'v>(mut self, iter: impl Iterator<Item = (&'k str, &'v str)>) -> Self {
        for (k, v) in iter {
            self.name_to_id.insert(v.into(), k.into());
            self.name_to_id.insert(k.into(), v.into());
            self.names.insert(k.into());
        }
        self
    }

    pub fn get_name(&self, id: &str) -> Option<&str> {
        self.id_to_name.get(id).map(|s| &**s)
    }

    pub fn get_id(&self, name: &str) -> Option<&str> {
        self.name_to_id.get(name).map(|s| &**s)
    }

    pub fn has(&self, name: &str) -> bool {
        self.name_to_id.contains_key(name)
    }

    pub fn names(&self) -> impl Iterator<Item = &str> + ExactSizeIterator + '_ {
        self.names.iter().map(|s| &**s)
    }
}

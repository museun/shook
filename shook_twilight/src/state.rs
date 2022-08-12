use std::{collections::HashMap, future::Future, sync::Arc};

use tokio::sync::Mutex;
use twilight_model::id::{marker::ChannelMarker, Id};

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct DiscordState {
    pub channels: Map<ChannelMarker>,
}

#[derive(Clone)]
pub struct Map<T, V = String> {
    map: Arc<Mutex<HashMap<Id<T>, Arc<V>>>>,
}

impl<T, V> Default for Map<T, V> {
    fn default() -> Self {
        Self {
            map: Default::default(),
        }
    }
}

impl<'de, T, V> serde::Deserialize<'de> for Map<T, V>
where
    V: serde::de::DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        <HashMap<Id<T>, V>>::deserialize(deserializer)
            .map(|s| s.into_iter().map(|(k, v)| (k, Arc::new(v))).collect())
            .map(Mutex::new)
            .map(Arc::new)
            .map(|map| Map { map })
    }
}

impl<T, V> serde::Serialize for Map<T, V>
where
    V: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap as _;
        let g = self.map.blocking_lock();
        let mut s = serializer.serialize_map(Some(g.len()))?;
        for (k, v) in g.iter() {
            s.serialize_entry(k, &**v)?
        }
        s.end()
    }
}

impl<T> Map<T> {
    pub async fn update<S, Fut>(
        &self,
        id: Id<T>,
        vacant: impl Fn() -> Fut + Send,
    ) -> anyhow::Result<Arc<String>>
    where
        S: Into<String> + Send,
        Fut: Future<Output = anyhow::Result<S>> + Send,
    {
        use std::collections::hash_map::Entry;
        match self.map.lock().await.entry(id) {
            Entry::Occupied(t) => Ok(t.get().clone()),
            Entry::Vacant(t) => {
                let data = vacant().await?.into();
                Ok(t.insert(Arc::new(data)).clone())
            }
        }
    }
}

use anyhow::Context;

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
};
use tokio::sync::{RwLock, RwLockReadGuard};

use crate::ConfigPath;

#[derive(Default, Clone)]
pub struct GlobalState(pub(crate) Arc<RwLock<State>>);

impl GlobalState {
    pub async fn get_config_path<C>(&self) -> PathBuf
    where
        C: ConfigPath + Any + Send + Sync,
    {
        self.get::<C>().await.file_path().to_path_buf()
    }
}

impl GlobalState {
    pub fn new(state: State) -> Self {
        Self(Arc::new(RwLock::new(state)))
    }

    // TODO this should use try_ and unwrap
    pub async fn get<T>(&self) -> RwLockReadGuard<'_, T>
    where
        T: Any + Send + Sync + 'static,
    {
        RwLockReadGuard::map(self.0.read().await, |state| state.get::<T>().unwrap())
    }

    pub async fn get_owned<T>(&self) -> T
    where
        T: Any + Send + Sync + 'static,
        T: Clone,
    {
        (&*self.get::<T>().await).clone()
    }

    pub async fn try_get_owned<T>(&self) -> Option<T>
    where
        T: Any + Send + Sync + 'static,
        T: Clone,
    {
        Some((&*self.try_get::<T>().await?).clone())
    }

    pub async fn try_get<T>(&self) -> Option<RwLockReadGuard<'_, T>>
    where
        T: Any + Send + Sync + 'static,
    {
        RwLockReadGuard::try_map(self.0.read().await, |state| state.get::<T>().ok()).ok()
    }

    pub async fn insert<T>(&self, val: T)
    where
        T: Any + Send + Sync + 'static,
    {
        self.0.write().await.insert(val);
    }
}

#[derive(Default, Debug)]
pub struct State {
    map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl State {
    pub fn get_config_path<C>(&self) -> PathBuf
    where
        C: ConfigPath + Any + Send + Sync,
    {
        self.get::<C>()
            .expect("config path")
            .file_path()
            .to_path_buf()
    }
}

impl State {
    pub fn insert<T>(&mut self, val: T)
    where
        T: Any + Send + Sync + 'static,
    {
        if let Some(..) = self.map.insert(TypeId::of::<T>(), Box::new(val)) {
            log::warn!("replaced: {}", std::any::type_name::<T>());
        }
    }

    pub fn get<T>(&self) -> anyhow::Result<&T>
    where
        T: Any + Send + Sync + 'static,
    {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|c| c.downcast_ref())
            .with_context(|| anyhow::anyhow!("could not find {}", Self::name_of::<T>()))
    }

    pub fn get_mut<T>(&mut self) -> anyhow::Result<&mut T>
    where
        T: Any + Send + Sync + 'static,
    {
        self.map
            .get_mut(&TypeId::of::<T>())
            .and_then(|c| c.downcast_mut())
            .with_context(|| anyhow::anyhow!("could not find {}", Self::name_of::<T>()))
    }

    pub fn extract<T, U, F>(&self, map: F) -> anyhow::Result<U>
    where
        T: Any + Send + Sync + 'static,
        U: 'static,
        F: FnOnce(&T) -> U,
    {
        self.get::<T>().map(map)
    }

    fn name_of<T: 'static>() -> &'static str {
        std::any::type_name::<T>()
    }
}

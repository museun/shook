use anyhow::Context;

#[derive(Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct Secret<T>(pub T);

impl<T> Secret<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl From<String> for Secret<String> {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl std::fmt::Debug for Secret<String> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, r#"{{len = {}}}"#, self.0.len())
    }
}

impl std::fmt::Display for Secret<String> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl<T> std::ops::Deref for Secret<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type Assign<T> = fn(&mut T, String);
pub fn load_from_env<T: Default + std::fmt::Debug>(
    keys: &[(&str, Assign<T>)],
) -> anyhow::Result<T> {
    let get = |key| {
        log::trace!("looking up {key}");
        let res = std::env::var(key);
        res.with_context(|| anyhow::anyhow!("key '{key}' was not found"))
    };

    log::trace!("loading env vars for: {}", std::any::type_name::<T>());

    let this = keys.iter().try_fold(T::default(), |mut this, (key, func)| {
        func(&mut this, get(key)?);
        Ok(this)
    });

    if let Ok(this) = &this {
        log::debug!("created: {:?}", this);
    }
    this
}

pub trait LoadFromEnv
where
    Self: Sized,
{
    fn load_from_env() -> anyhow::Result<Self>;
}

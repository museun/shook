use anyhow::Context;

mod secret;
pub use secret::Secret;

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

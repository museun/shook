use std::borrow::Cow;

#[derive(Clone)]
pub struct Secret {
    inner: Cow<'static, str>,
    key: Cow<'static, str>,
}

impl Secret {
    pub fn key(key: &str) -> Self {
        Self {
            inner: Cow::default(),
            key: Cow::from(key.to_string()),
        }
    }

    pub fn env_key(&self) -> &str {
        &self.key
    }

    pub fn inner(&self) -> &str {
        &self.inner
    }

    pub fn into_string(self) -> String {
        self.inner.to_string()
    }
}

impl AsRef<str> for Secret {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl std::ops::Deref for Secret {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Secret")
            .field("inner", &crate::redact(&self.inner))
            .field("key", &self.key)
            .finish()
    }
}

impl serde::Serialize for Secret {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.key)
    }
}

impl<'de> serde::Deserialize<'de> for Secret {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let key = <Cow<'_, str>>::deserialize(deserializer)?;
        let inner = std::env::var(&*key)
            .map_err(D::Error::custom)
            .map(Cow::from)?;
        Ok(Self { inner, key })
    }
}

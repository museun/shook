use super::{AnyhowFut, Persist, PersistFormat};
use tokio::fs::File;

type Path<'a> = dyn AsRef<std::path::Path> + Send + Sync + 'a;

pub trait PersistExt: Persist + Send + Sync {
    fn save_to_file<'a, K>(&'a self, path: &'a Path<'a>) -> AnyhowFut<'a, ()>
    where
        K: PersistFormat,
    {
        let path = K::with_ext(path.as_ref());
        Box::pin(async move {
            let mut file = File::create(path).await?;
            self.save::<K>(&mut file).await
        })
    }

    fn load_from_file<'a, K>(path: &'a Path<'a>) -> AnyhowFut<'a, Self>
    where
        K: PersistFormat,
    {
        let path = K::with_ext(path.as_ref());
        Box::pin(async move {
            let mut file = File::open(path).await?;
            Self::load::<K>(&mut file).await
        })
    }
}

impl<T> PersistExt for T where T: Persist + 'static {}

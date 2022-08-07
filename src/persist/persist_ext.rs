use super::{AnyhowFut, Persist, PersistFormat};
use std::path::Path;
use tokio::fs::File;

pub trait PersistExt: Persist + Send + Sync {
    fn save_to_file<'a, K: PersistFormat>(
        &'a self,
        path: &'a (dyn AsRef<Path> + Send + Sync + 'a),
    ) -> AnyhowFut<'a, ()> {
        let path = K::with_ext(path.as_ref());
        Box::pin(async move {
            let mut file = File::create(path).await?;
            self.save::<K>(&mut file).await
        })
    }

    fn load_from_file<'a, K: PersistFormat>(
        path: &'a (dyn AsRef<Path> + Sync + Send + 'a),
    ) -> AnyhowFut<'a, Self> {
        let path = K::with_ext(path.as_ref());
        Box::pin(async move {
            let mut file = File::open(path).await?;
            Self::load::<K>(&mut file).await
        })
    }
}

impl<T: Send + Sync + 'static> PersistExt for T where T: Persist {}

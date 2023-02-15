use {
    async_trait::async_trait,
    super::resolver::DataResolver,
};

pub struct CachedResolver<T: DataResolver, E: DataResolver> {
    inner: T,
    cache: E,
}

impl <T: DataResolver, E: DataResolver> CachedResolver<T, E> {
    pub fn new(inner: T, cache: E) -> Self {
        Self {
            inner,
            cache,
        }
    }
}

#[async_trait]
impl <T: DataResolver + Send + Sync, E: DataResolver + Send + Sync> DataResolver for CachedResolver<T, E> {
    async fn resolve(&self, key: &str) -> Option<Vec<u8>> {
        unimplemented!()
    }

    async fn resolve_to_fs_path(&self, key: &str) -> Option<String> {
        unimplemented!()
    }
}
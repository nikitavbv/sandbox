use {
    tracing::info,
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
        if let Some(cached_value) = self.cache.resolve(key).await {
            return Some(cached_value);
        }

        info!("downloading {} to cache", key);
        let new_value = self.inner.resolve(key).await.unwrap();
        self.cache.put(key, new_value.clone()).await;
        Some(new_value)
    }

    async fn resolve_to_fs_path(&self, key: &str) -> Option<String> {
        info!("trying to resolve {}", key);

        if let Some(cached_value) = self.cache.resolve_to_fs_path(key).await {
            return Some(cached_value);
        }

        info!("downloading {} to cache", key);
        let new_value = self.inner.resolve(key).await.unwrap();
        self.cache.put(key, new_value).await;

        self.cache.resolve_to_fs_path(key).await
    }
}
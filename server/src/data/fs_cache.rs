use {
    async_trait::async_trait,
    super::resolver::DataResolver,
};

pub struct FileSystemCachedResolver<T: DataResolver> {
    inner: T,
}

impl <T: DataResolver> FileSystemCachedResolver<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
        }
    }
}

#[async_trait]
impl <T: DataResolver + Send + Sync> DataResolver for FileSystemCachedResolver<T> {
    async fn resolve(&self, key: &str) -> Option<Vec<u8>> {
        self.inner.resolve(key).await
    }

    async fn resolve_to_fs_path(&self, key: &str) -> Option<String> {
        self.inner.resolve_to_fs_path(key).await
    }
}
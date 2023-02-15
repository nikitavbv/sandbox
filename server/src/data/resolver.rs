#[async_trait::async_trait]
pub trait DataResolver {
    async fn resolve(&self, key: &str) -> Option<Vec<u8>>;
    async fn resolve_to_fs_path(&self, key: &str) -> Option<String>;
    async fn put(&self, key: &str, value: Vec<u8>) {
    }
}
#[async_trait::async_trait]
pub trait DataResolver {
    async fn resolve(&self, key: &str) -> Vec<u8>;
    async fn resolve_to_fs_path(&self, key: &str) -> String;
}
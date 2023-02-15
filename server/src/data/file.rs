use {
    std::path::Path,
    tokio::fs,
    super::resolver::DataResolver,
};

pub struct FileDataResolver {
    directory: String,
}

impl FileDataResolver {
    pub fn new(directory: String) -> Self {
        Self {
            directory,
        }
    }

    pub fn path_for_key(&self, key: &str) -> String {
        let path = Path::new(&self.directory);
        path.join(key).to_string_lossy().to_string()
    }
}

#[async_trait::async_trait]
impl DataResolver for FileDataResolver {
    async fn resolve(&self, key: &str) -> Vec<u8> {
        fs::read(&self.path_for_key(key)).await.unwrap()
    }

    async fn resolve_to_fs_path(&self, key: &str) -> String {
        self.path_for_key(key)
    }
}
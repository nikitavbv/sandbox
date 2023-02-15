use {
    tracing::info,
    awsregion::Region,
    s3::{Bucket, creds::Credentials},
    config::Config,
    async_trait::async_trait,
    super::resolver::DataResolver,
};

pub struct ObjectStorageDataResolver {
    bucket: Bucket,
    prefix: String,
}

impl ObjectStorageDataResolver {
    pub fn new_with_config(bucket_name: String, prefix: String, config: &Config) -> Self {
        Self::new(
            config.get("object_storage.endpoint").unwrap(),
            config.get("object_storage.region").unwrap(),
            config.get("object_storage.access_key").unwrap(),
            config.get("object_storage.secret_key").unwrap(),
            bucket_name,
            prefix
        )
    }

    pub fn new(endpoint: String, region: String, access_key: String, secret_key: String, bucket_name: String, prefix: String) -> Self {
        Self {
            bucket: Bucket::new(
                &bucket_name,
                Region::Custom {
                    region,
                    endpoint,
                },
                Credentials::new(Some(&access_key), Some(&secret_key), None, None, None).unwrap()
            ).unwrap().with_path_style(),
            prefix,
        }
    }
}

#[async_trait]
impl DataResolver for ObjectStorageDataResolver {
    async fn resolve(&self, key: &str) -> Option<Vec<u8>> {
        let path = format!("{}/{}", self.prefix, key);
        Some(self.bucket.get_object(path).await.unwrap().to_vec())
    }

    async fn resolve_to_fs_path(&self, key: &str) -> Option<String> {
        None
    }
}
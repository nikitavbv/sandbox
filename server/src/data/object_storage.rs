use {
    awsregion::Region,
    s3::{Bucket, creds::Credentials},
    super::resolver::DataResolver,
};

pub struct ObjectStorageDataResolver {
    bucket: Bucket,
}

impl ObjectStorageDataResolver {
    pub fn new(endpoint: String, region: String, bucket_name: String, access_key: String, secret_key: String) -> Self {
        Self {
            bucket: Bucket::new(
                &bucket_name,
                Region::Custom {
                    region,
                    endpoint,
                },
                Credentials::new(Some(&access_key), Some(&secret_key), None, None, None).unwrap()
            ).unwrap().with_path_style(),
        }
    }
}

#[async_trait::async_trait]
impl DataResolver for ObjectStorageDataResolver {
    async fn resolve(&self, key: &str) -> Vec<u8> {
        unimplemented!()
    }

    async fn resolve_to_fs_path(&self, key: &str) -> String {
        unimplemented!()
    }
}
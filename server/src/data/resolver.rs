use {
    std::{
        path::Path,
        time::Duration,
    },
    tokio::fs,
    awsregion::Region,
    s3::{Bucket, creds::Credentials},
    config::Config,
};

pub struct DataResolver {
    bucket: Option<Bucket>,
    bucket_prefix: String,
}

impl DataResolver {
    pub fn new(config: &Config) -> Self {
        let endpoint = config.get("object_storage.endpoint").unwrap();
        let region = config.get("object_storage.region").unwrap();
        let access_key: String = config.get("object_storage.access_key").unwrap();
        let secret_key: String = config.get("object_storage.secret_key").unwrap();
        let bucket_name = "nikitavbv-sandbox";
        let bucket_prefix = "data".to_owned(); 
        
        let bucket = Bucket::new(
            &bucket_name,
            Region::Custom {
                region,
                endpoint,
            },
            Credentials::new(Some(&access_key), Some(&secret_key), None, None, None).unwrap(),
        ).unwrap().with_path_style().with_request_timeout(Duration::from_secs(60 * 20));

        Self {
            bucket: Some(bucket),
            bucket_prefix,
        }
    }
    
    pub async fn resolve(&self, key: &str) -> Option<Vec<u8>> {
        let path = self.resolve_to_fs_path(key).await;

        if let Some(path) = path {
            Some(fs::read(path).await.unwrap())
        } else {
            None
        }
    }

    pub async fn resolve_to_fs_path(&self, key: &str) -> Option<String> {
        let path_str = self.key_to_path(key);
        let path = Path::new(&path_str);
        if path.exists() {
            return Some(path_str);
        }

        let bucket_path = format!("{}/{}", self.bucket_prefix, key);
        let object_data = match self.bucket.as_ref().unwrap().get_object(&bucket_path).await {
            Ok(v) => v.to_vec(),
            Err(err) => panic!("failed to fetch object with path: {}, error is: {}", bucket_path, err),
        };

        fs::write(path, object_data).await.unwrap();

        None
    }

    pub async fn put(&self, path: String, value: &[u8]) {
        self.bucket.as_ref().unwrap().put_object(path, value).await.unwrap();
    }

    fn key_to_path(&self, key: &str) -> String {
        Path::new("data").join(key).to_string_lossy().to_string()
    }
}
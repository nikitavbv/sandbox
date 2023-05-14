use {
    s3::{Bucket, creds::Credentials, region::Region},
    config::Config,
};

pub struct Storage {
    bucket: s3::Bucket,
}

impl Storage {
    pub fn new(config: &Config) -> Self {
        let region = config.get_string("object_storage.region").unwrap();
        let endpoint = config.get_string("object_storage.endpoint").unwrap();
        let access_key = config.get_string("object_storage.access_key").unwrap();
        let secret_key = config.get_string("object_storage.secret_key").unwrap();

        let bucket = Bucket::new(
            "sandbox",
            Region::Custom {
                region,
                endpoint,
            },
            Credentials::new(Some(&access_key), Some(&secret_key), None, None, None).unwrap(),
        ).unwrap().with_path_style();

        Self {
            bucket,
        }
    }

    pub async fn get_generated_image(&self, task_id: &str) -> Vec<u8> {
        let key = format!("output/images/{}", task_id);
        self.bucket.get_object(&key).await.unwrap().to_vec()
    }
}
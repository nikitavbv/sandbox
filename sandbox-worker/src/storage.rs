use {
    std::{fs, path::Path},
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

    pub async fn load_model_file(&self, file_name: &str) -> String {
        let model_data_dir = Path::new("data/model/stable-diffusion");
        if !model_data_dir.exists() {
            fs::create_dir_all(&model_data_dir).unwrap();
        }
        
        let file_path = model_data_dir.join(file_name);
        let file_path_str = file_path.to_str().unwrap().to_owned();
        if file_path.exists() {
            return file_path_str;
        }

        let res = self.bucket.get_object(&format!("data/model/stable-diffusion/{}", file_name)).await.unwrap();
        tokio::fs::write(file_path, res.as_slice()).await.unwrap();

        file_path_str
    }

    pub async fn save_generated_image(&self, task_id: &str, image: &[u8]) {
        let key = format!("output/images/{}", task_id);
        self.bucket.put_object(&key, image).await.unwrap();
    }
}
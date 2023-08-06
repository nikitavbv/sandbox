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

    pub async fn load_model_file(&self, model_name: &str, file_name: &str) -> String {
        let model_data_dir = format!("data/model/{}", model_name);
        let model_data_dir = Path::new(&model_data_dir);
        if !model_data_dir.exists() {
            fs::create_dir_all(&model_data_dir).unwrap();
        }
        
        let file_path = model_data_dir.join(file_name);
        let file_path_str = file_path.to_str().unwrap().to_owned();
        if file_path.exists() {
            return file_path_str;
        }

        self.bucket.get_object_to_writer(&format!("model/{}/{}", model_name, file_name), &mut tokio::fs::File::create(file_path).await.unwrap()).await.unwrap();

        file_path_str
    }
}
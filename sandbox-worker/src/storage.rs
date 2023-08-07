use {
    std::{fs, path::Path},
    tracing::info,
    s3::{Bucket, creds::Credentials, region::Region},
    config::Config,
    tokio::io::AsyncWriteExt,
    indicatif::ProgressBar,
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

        info!("downloading file \"{}\" for model {}", file_name, model_name);

        let mut file = tokio::fs::File::create(file_path).await.unwrap();
        
        let key = format!("model/{}/{}", model_name, file_name);
        let head = self.bucket.head_object(&key).await.unwrap();
        let file_size = head.0.content_length.unwrap() as usize;

        let progress = ProgressBar::new(file_size as u64);

        let block_size = 10 * 1024 * 1024; // 10 megabytes
        let mut i = 0;
        while i < file_size {
            let block = self.bucket.get_object_range(&key, i as u64, Some((i + block_size) as u64)).await.unwrap();
            let block = block.as_slice();
            i += block.len();
            progress.inc(block.len() as u64);
            file.write(block).await.unwrap();
        }
        progress.finish_and_clear();

        info!("finished downloading file \"{}\"", file_name);

        file_path_str
    }
}
use {
    std::{
        collections::HashMap,
        sync::Arc,
    },
    tokio::sync::Mutex,
    config::Config,
    crate::{
        data::{
            file::FileDataResolver,
            cached_resolver::CachedResolver,
            object_storage::ObjectStorageDataResolver,
        },
        models::{
            Model,
            image_generation::StableDiffusionImageGenerationModel,
        },
    },
};

pub struct ModelRegistry {
    models: HashMap<String, Arc<Mutex<Box<dyn Model + Send>>>>,
}

impl ModelRegistry {
    pub async fn new(config: &Config) -> Self {
        let object_storage_resolver = ObjectStorageDataResolver::new_with_config(
            "nikitavbv-sandbox".to_owned(), 
            "data/models/stable-diffusion".to_owned(), 
            config
        );

        let file_resolver = FileDataResolver::new("./data/stable-diffusion".to_owned());
        let data_resolver = CachedResolver::new(object_storage_resolver, file_resolver);

        let mut models: HashMap<String, Arc<Mutex<Box<dyn Model + Send>>>> = HashMap::new();        
        // models.insert("image_generation".to_owned(), Arc::new(Mutex::new(Box::new(StableDiffusionImageGenerationModel::new(&data_resolver).await))));

        Self {
            models,
        }
    }

    pub fn add_model(&mut self, model_id: String, model: Box<dyn Model + Send>) {
        self.models.insert(model_id, Arc::new(Mutex::new(model)));
    }

    pub fn get(&self, model_id: &str) -> Option<Arc<Mutex<Box<dyn Model + Send>>>> {
        self.models.get(model_id).cloned()
    }
}
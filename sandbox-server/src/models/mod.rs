use {
    std::{fs::File, io::BufReader, pin::Pin, future::Future, sync::Arc},
    tracing::info,
    npyz::npz::NpzArchive,
    config::Config,
    image::{DynamicImage, imageops::FilterType, GenericImageView},
    tokio::sync::Mutex,
    crate::context::Context,
    self::io::ModelData,
};

#[cfg(feature="tch-inference")]
pub mod image_generation;
#[cfg(feature="tch-inference")]
pub mod text_summarization;
#[cfg(feature="tch-inference")]
pub mod text_generation;

pub mod io;

pub trait Model {
    fn create() -> Self where Self:Sized {
        unimplemented!()
    }
    
    fn run(&self, input: &ModelData) -> ModelData;
}

#[derive(Clone)]
pub struct ModelDefinition {
    id: String,
    factory: fn(Arc<Context>) -> Pin<Box<dyn Future<Output = Box<dyn Model + Send>> + Send>>,
}

impl ModelDefinition {
    pub fn new(id: String, factory: fn(Arc<Context>) -> Pin<Box<dyn Future<Output = Box<dyn Model + Send>> + Send>>) -> Self {
        Self {
            id,
            factory,
        }
    }

    pub async fn create_instance(&self, context: Arc<Context>) -> Box<dyn Model + Send> {
        (self.factory)(context).await
    }

    pub fn id(&self) -> &String {
        &self.id
    }
}

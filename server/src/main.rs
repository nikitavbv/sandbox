use {
    std::{sync::Arc, pin::Pin, future::Future},
    tracing::{info, error},
    config::Config,
    crate::{
        utils::init_logging,
        server::run_axum_server,
        labeling::run_data_labeling_tasks,
        models::{
            Model,
            ModelDefinition,
            run_simple_model_inference, 
            image_generation::{StableDiffusionImageGenerationModel, run_simple_image_generation},
            text_generation::{TextGenerationModel, run_simple_text_generation},
            text_summarization::{TextSummarizationModel, run_simple_text_summarization},
        },
        scheduling::{
            scheduler::Scheduler,
            simple::SimpleScheduler,
            registry::ModelRegistry,
        },
        context::Context,
    },
};

pub mod data;
pub mod labeling;
pub mod models;
pub mod scheduling;
pub mod context;
pub mod server;
pub mod utils;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_logging();

    let config = Config::builder()
        .add_source(config::File::with_name("./config.toml"))
        .add_source(config::Environment::with_prefix("SANDBOX"))
        .build()
        .unwrap();

    match config.get_string("target").unwrap_or("server".to_owned()).as_str() {
        "server" => run_axum_server(&config, create_scheduler(&config).await).await,
        "simple_model" => run_simple_model_inference(),
        "simple_image_generation" => run_simple_image_generation(&config).await,
        "simple_text_generation" => run_simple_text_generation().await,
        "simple_text_summarization" => run_simple_text_summarization().await,
        "data_labeling" => run_data_labeling_tasks(&config),
        other => error!("Unexpected action: {}", other),
    };

    info!("done");
    Ok(())
}

async fn create_scheduler(config: &Config) -> impl Scheduler {
    let registry = ModelRegistry::new(config).await
        .with_definition(ModelDefinition::new("image-generation".to_owned(), image_generation_model_factory))
        .with_definition(ModelDefinition::new("text-generation".to_owned(), text_generation_model_factory))
        .with_definition(ModelDefinition::new("text-summarization".to_owned(), text_summarization_model_factory));

    SimpleScheduler::new(registry)
}

fn image_generation_model_factory(context: Arc<Context>) -> Pin<Box<dyn Future<Output = Box<dyn Model + Send>> + Send>> {    
    Box::pin(async move {
        Box::new(StableDiffusionImageGenerationModel::new(&context.data_resolver()).await) as Box<dyn Model + Send>
    })
}

fn text_generation_model_factory(_context: Arc<Context>) -> Pin<Box<dyn Future<Output = Box<dyn Model + Send>> + Send>> {
    Box::pin(async move {
        Box::new(TextGenerationModel::new()) as Box<dyn Model + Send>
    })
}

fn text_summarization_model_factory(_context: Arc<Context>) -> Pin<Box<dyn Future<Output = Box<dyn Model + Send>> + Send>> {
    Box::pin(async move {
        Box::new(TextSummarizationModel::new()) as Box<dyn Model + Send>
    })
}
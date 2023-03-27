use {
    std::{sync::Arc, pin::Pin, future::Future, env::var},
    tracing::{info, error},
    config::Config,
    crate::{
        utils::init_logging,
        server::run_axum_server,
        autoscaling::shutdown::AutoShutdownScheduler,
        data::resolver::DataResolver,
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
            nop::DoNothingScheduler,
            pg_queue::{PgQueueSchedulerClient, PgQueueWorker},
        },
        context::Context,
    },
};

#[cfg(feature = "video-hashes")]
pub mod labeling;
#[cfg(feature = "video-hashes")]
use crate::labeling::run_data_labeling_tasks;

pub mod autoscaling;
pub mod data;
pub mod models;
pub mod scheduling;
pub mod context;
pub mod server;
pub mod utils;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_logging();

    let config = Config::builder()
        .add_source(config::File::with_name(var("SANDBOX_CONFIG_PATH").unwrap_or("./config.toml".to_owned()).as_str()))
        .add_source(config::Environment::with_prefix("SANDBOX"))
        .build()
        .unwrap();

    match config.get_string("target").unwrap_or("server".to_owned()).as_str() {
        "server" => run_axum_server(&config, init_scheduler(&config).await).await,
        "simple_model" => run_simple_model_inference(),
        "simple_image_generation" => run_simple_image_generation(&config).await,
        "simple_text_generation" => run_simple_text_generation().await,
        "simple_text_summarization" => run_simple_text_summarization().await,
        "data_labeling" => run_data_labeling_tasks(&config),
        "worker" => run_worker(&config).await,
        other => error!("Unexpected action: {}", other),
    };

    info!("done");
    Ok(())
}

#[cfg(not(feature = "video-hashes"))]
fn run_data_labeling_tasks(_config: &Config) {
    panic!("server was built without support for video-hashes features");
}

async fn init_scheduler(config: &Config) -> Box<dyn Scheduler + Send + Sync> {
    let scheduler_name = config.get_string("scheduler.name").unwrap_or("simple".into());

    info!("using scheduler: {}", scheduler_name);

    let mut scheduler = match scheduler_name.as_str() {
        "simple" => init_simple_scheduler(config).await,
        "nop" => Box::new(DoNothingScheduler::new()),
        "pg_queue" => init_pg_queue_scheduler(config).await,
        other => panic!("unknown scheduler: {}", other),
    };

    if config.get_bool("scheduler.auto_shutdown").unwrap_or(false) {
        scheduler = Box::new(AutoShutdownScheduler::new(scheduler));
    }

    scheduler
}

async fn run_worker(config: &Config) {
    let context = Context::new(DataResolver::new(config));

    let worker = PgQueueWorker::new(
        &config
            .get_string("worker.postgres_connection_string")
            .unwrap(),
        init_scheduler(config).await,
        context,
    )
    .await;

    worker.run().await;
}

async fn init_simple_scheduler(config: &Config) -> Box<dyn Scheduler + Send + Sync> {
    let registry = ModelRegistry::new(config).await
        .with_definition(ModelDefinition::new("image-generation".to_owned(), image_generation_model_factory))
        .with_definition(ModelDefinition::new("text-generation".to_owned(), text_generation_model_factory))
        .with_definition(ModelDefinition::new("text-summarization".to_owned(), text_summarization_model_factory));

    Box::new(SimpleScheduler::new(registry))
}

async fn init_pg_queue_scheduler(config: &Config) -> Box<dyn Scheduler + Send + Sync> {
    Box::new(PgQueueSchedulerClient::new(&config.get_string("scheduler.postgres_connection_string").unwrap()).await)
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
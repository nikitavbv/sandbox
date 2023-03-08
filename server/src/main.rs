use {
    tracing::{info, error},
    config::Config,
    crate::{
        utils::init_logging,
        server::run_axum_server,
        labeling::run_data_labeling_tasks,
        models::{
            run_simple_model_inference, 
            image_generation::run_simple_image_generation,
            text_generation::run_simple_text_generation,
            text_summarization::run_simple_text_summarization,
        },
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
        "server" => run_axum_server(&config).await,
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

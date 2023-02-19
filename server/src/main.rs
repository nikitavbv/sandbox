#![feature(async_closure)]

use {
    tracing::info,
    config::Config,
    crate::{
        utils::init_logging,
        server::run_server,
        models::{run_simple_model_inference, image_generation::run_simple_image_generation},
    },
};

pub mod data;
pub mod models;
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

    // run_server(&config).await;
    // run_simple_model_inference();
    run_simple_image_generation(&config).await;

    info!("done");
    Ok(())
}

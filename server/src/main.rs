#![feature(async_closure)]

use {
    tracing::info,
    crate::{
        utils::init_logging,
        server::run_server,
        models::{run_simple_model_inference, image_generation::run_simple_image_generation},
    },
};

pub mod models;
pub mod server;
pub mod utils;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_logging();

    // run_server().await;
    // run_simple_model_inference();
    run_simple_image_generation();

    info!("done");
    Ok(())
}

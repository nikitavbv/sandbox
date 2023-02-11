#![feature(async_closure)]

use {
    tracing::info,
    crate::{
        utils::init_logging,
        server::run_server,
        models::run_simple_model_inference,
    },
};

pub mod models;
pub mod server;
pub mod utils;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_logging();

    run_server().await;
    // run_simple_model_inference();

    info!("done");
    Ok(())
}

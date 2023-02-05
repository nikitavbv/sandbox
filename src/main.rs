#![feature(async_closure)]

use {
    std::sync::Arc,
    crate::{
        server::run_http_server,
        utils::init_logging,
        runner::runner,
    },
};

pub mod models;
pub mod runner;
pub mod server;
pub mod utils;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_logging();
    run_http_server(Arc::new(runner)).await;
    Ok(())
}

#![feature(async_closure)]

use {
    crate::{
        utils::init_logging,
        server::run_server,
    },
};

pub mod models;
pub mod server;
pub mod utils;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_logging();

    run_server().await;

    Ok(())
}

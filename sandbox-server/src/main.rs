use {
    std::env::var,
    tracing::{info, error},
    config::Config,
    sandbox_common::utils::{init_logging, load_config},
    crate::{
        server::run_server,
    },
};

// All logic lives in services and handlers are connecting it to the outside world using data structures defined in entities.

pub mod entities;
pub mod handlers;
pub mod services;
pub mod state;
pub mod server;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_logging();

    let config = load_config();

    run_server(&config).await;

    info!("done");
    Ok(())
}

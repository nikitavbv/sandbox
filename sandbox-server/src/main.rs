use {
    std::env::var,
    tracing::{info, error},
    config::Config,
    sandbox_common::utils::{init_logging, load_config},
    crate::{
        server::run_server,
    },
};

pub mod entities;
pub mod handlers;
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

use {
    tracing::info,
    crate::{
        server::run_server,
        worker::run_worker,
        utils::{init_logging, load_config},
    },
};

pub mod entities;
pub mod handlers;
pub mod state;
pub mod worker;
pub mod server;
pub mod utils;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_logging();

    let config = load_config();

    if config.get_bool("server.enabled").unwrap_or(true) {
        run_server(&config).await;
    }

    if config.get_bool("worker.enabled").unwrap_or(true) {
        run_worker(&config).await;
    }

    info!("done");
    Ok(())
}

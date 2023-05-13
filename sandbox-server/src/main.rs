use {
    std::env::var,
    tracing::{info, error},
    config::Config,
    crate::{
        utils::init_logging,
        server::run_axum_server,
    },
};

pub mod autoscaling;
pub mod state;

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

    run_axum_server(&config).await;

    info!("done");
    Ok(())
}

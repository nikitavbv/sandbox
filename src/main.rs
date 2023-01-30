use {
    server::run_http_server,
    utils::init_logging,
};

pub mod server;
pub mod utils;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_logging();
    run_http_server().await;
    Ok(())
}

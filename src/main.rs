#![feature(async_closure)]

use {
    std::sync::Arc,
    hyper::{Response, Body},
    crate::{
        server::run_http_server,
        utils::init_logging,
    },
};

pub mod models;
pub mod server;
pub mod utils;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_logging();
    run_http_server(Arc::new(async move |req| Response::new(Body::from("hello from handler!".to_owned())))).await;
    Ok(())
}

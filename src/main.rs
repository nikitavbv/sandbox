#![feature(async_closure)]

use {
    std::sync::Arc,
    hyper::Method,
    crate::{
        server::run_http_server,
        utils::init_logging,
        runner::runner,
        filters::HttpMethodFilter,
    },
};

pub mod filters;
pub mod models;
pub mod runner;
pub mod server;
pub mod utils;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_logging();
    run_http_server(Arc::new(HttpMethodFilter::new(Method::POST, runner))).await;
    Ok(())
}

#![feature(async_closure)]

use {
    actix_web::{HttpServer, App},
    crate::{
        utils::init_logging,
        app::init,
    },
};

pub mod app;
pub mod models;
pub mod runner;
pub mod utils;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_logging();

    HttpServer::new(|| {
        App::new()
            .configure(init)
    })
        .bind(("0.0.0.0", 8080))
        .unwrap()
        .run()
        .await
}

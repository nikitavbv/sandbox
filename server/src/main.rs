#![feature(async_closure)]

use {
    std::sync::Arc,
    actix_web::{HttpServer, App},
    crate::{
        utils::init_logging,
        app::AiLabApp,
    },
};

pub mod app;
pub mod models;
pub mod runner;
pub mod utils;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_logging();
    
    let app = Arc::new(AiLabApp::new());

    HttpServer::new(move || {
        App::new()
            .configure(|v| app.init(v))
    })
        .bind(("0.0.0.0", 8080))
        .unwrap()
        .run()
        .await
}

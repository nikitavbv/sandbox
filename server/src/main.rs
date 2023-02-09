#![feature(async_closure)]

use {
    std::sync::Arc,
    tonic::{transport::Server, Status, Request, Response},
    rpc::{
        ml_sandbox_service_server::{MlSandboxService, MlSandboxServiceServer},
        RunSimpleModelRequest,
        RunSimpleModelResponse,
    },
    crate::utils::init_logging,
};

pub mod models;
pub mod utils;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_logging();
    
    Server::builder()
        .accept_http1(true)
        .add_service(tonic_web::enable(MlSandboxServiceServer::new(MlSandboxServiceHandler)))
        .serve("0.0.0.0:8080".parse().unwrap())
        .await
        .unwrap();

    Ok(())
}

struct MlSandboxServiceHandler;

#[tonic::async_trait]
impl MlSandboxService for MlSandboxServiceHandler {
    async fn run_simple_model(&self, req: Request<RunSimpleModelRequest>) -> Result<Response<RunSimpleModelResponse>, Status> {
        Ok(Response::new(RunSimpleModelResponse {}))
    }
}
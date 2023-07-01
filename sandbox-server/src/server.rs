use {
    tonic::{Status, Request, Response},
    tracing::info,
    config::Config,
    rand::distributions::{Alphanumeric, Distribution},
    axum::{Router, routing::get},
    axum_tonic::{NestTonic, RestGrpcService},
    anyhow::Result,
    jsonwebtoken::{DecodingKey, Validation, Algorithm},
    serde::Deserialize,
    futures::join,
    tonic::transport::Server,
    rpc::{
        sandbox_service_server::{SandboxService, SandboxServiceServer},
        FILE_DESCRIPTOR_SET,
        GenerateImageRequest,
        GenerateImageResponse,
        TaskId,
        TaskStatus,
        HistoryRequest,
        TaskHistory,
        GetTaskToRunRequest,
        GetTaskToRunResponse,
        TaskToRun,
        UpdateTaskStatusRequest,
        UpdateTaskStatusResponse,
        Status as RpcStatus,
    },
    crate::{
        state::{
            database::{Database, Task}, 
            storage::Storage,
        },
        handlers::SandboxServiceHandler,
    },
};

pub async fn run_server(config: &Config) {
    let axum_server = run_axum_server(config);
    let grpc_server = run_grpc_server(config);
    join!(axum_server, grpc_server);
}

pub async fn run_axum_server(config: &Config) {
    let host = config.get_string("server.host").unwrap_or("0.0.0.0".to_owned());
    let port = config.get_int("server.port").unwrap_or(8080);
    let addr = format!("{}:{}", host, port).parse().unwrap();

    info!("starting axum server on {:?}", addr);
    
    axum::Server::bind(&addr)
        .serve(service(config).await.unwrap().into_make_service())
        .await
        .unwrap();
}

pub async fn run_grpc_server(config: &Config) {
    let port = config.get_int("server.grpc_port").unwrap_or(8081);

    Server::builder()
        .add_service(SandboxServiceServer::new(SandboxServiceHandler::new(config).await.unwrap()))
        .serve(format!("0.0.0.0:{}", port).parse().unwrap())
        .await
        .unwrap();
}

pub async fn service(config: &Config) -> Result<RestGrpcService> {
    let grpc = Router::new().nest("/api", grpc_router(config).await?);
    Ok(RestGrpcService::new(Router::new(), grpc))
}

async fn grpc_router(config: &Config) -> Result<Router> {
    Ok(Router::new()
        .nest_tonic(
            tonic_reflection::server::Builder::configure()
                .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
                .build()
                .unwrap()
        )
        .nest_tonic(tonic_web::enable(SandboxServiceServer::new(SandboxServiceHandler::new(config).await?))))
}

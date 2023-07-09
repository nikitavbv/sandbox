use {
    std::sync::Arc,
    tracing::info,
    config::Config,
    axum::Router,
    axum_tonic::{NestTonic, RestGrpcService},
    anyhow::Result,
    futures::join,
    tonic::transport::Server,
    rpc::{
        sandbox_service_server::SandboxServiceServer,
        FILE_DESCRIPTOR_SET,
    },
    crate::{
        handlers::{SandboxServiceHandler, rest::{OAuthClientSecret, rest_router}},
        state::database::Database,
    },
};

pub async fn run_server(config: &Config) {
    let database = Arc::new(Database::new(config, &config.get_string("database.connection_string").unwrap()).await.unwrap());
    let axum_server = run_axum_server(config, database.clone());
    let grpc_server = run_grpc_server(config, database);
    join!(axum_server, grpc_server);
}

pub async fn run_axum_server(config: &Config, database: Arc<Database>) {
    let host = config.get_string("server.host").unwrap_or("0.0.0.0".to_owned());
    let port = config.get_int("server.port").unwrap_or(8081);
    let addr = format!("{}:{}", host, port).parse().unwrap();
    
    let oauth_client_secret = OAuthClientSecret { client_secret: config.get("auth.oauth_client_secret").unwrap() };
    let encoding_key = jsonwebtoken::EncodingKey::from_rsa_pem(config.get_string("auth.encoding_key").unwrap().as_bytes()).unwrap();

    info!("starting axum server on {:?}", addr);
    
    axum::Server::bind(&addr)
        .serve(service(config, database, oauth_client_secret, encoding_key).await.unwrap().into_make_service())
        .await
        .unwrap();
}

pub async fn run_grpc_server(config: &Config, database: Arc<Database>) {
    let port = config.get_int("server.grpc_port").unwrap_or(8082);
    let addr = format!("0.0.0.0:{}", port).parse().unwrap();

    info!("starting grpc server on port {:?}", addr);

    Server::builder()
        .add_service(SandboxServiceServer::new(SandboxServiceHandler::new(config, database).await.unwrap()))
        .serve(addr)
        .await
        .unwrap();
}

pub async fn service(
    config: &Config, 
    database: Arc<Database>, 
    oauth_client_secret: OAuthClientSecret,
    encoding_key: jsonwebtoken::EncodingKey,
) -> Result<RestGrpcService> {
    let grpc = Router::new().nest("/v1/rpc", grpc_router(config, database.clone()).await?);
    let rest = rest_router(database, oauth_client_secret, encoding_key);
    Ok(RestGrpcService::new(rest, grpc))
}

async fn grpc_router(config: &Config, database: Arc<Database>) -> Result<Router> {
    Ok(Router::new()
        .nest_tonic(
            tonic_reflection::server::Builder::configure()
                .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
                .build()
                .unwrap()
        )
        .nest_tonic(tonic_web::enable(SandboxServiceServer::new(SandboxServiceHandler::new(config, database).await?))))
}

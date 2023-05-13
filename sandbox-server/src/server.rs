use {
    tonic::{Status, Request, Response},
    tracing::info,
    config::Config,
    rand::distributions::{Alphanumeric, Distribution},
    axum::{Router, routing::get},
    axum_tonic::{NestTonic, RestGrpcService},
    anyhow::Result,
    rpc::{
        ml_sandbox_service_server::{MlSandboxService, MlSandboxServiceServer},
        FILE_DESCRIPTOR_SET,
        GenerateImageRequest,
        GenerateImageResponse,
    },
    crate::state::{database::Database, queue::{Queue, TaskMessage}},
};

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

pub async fn service(config: &Config) -> Result<RestGrpcService> {
    let app = rest_router();
    let grpc = Router::new().nest("/api", grpc_router(config).await?);
    Ok(RestGrpcService::new(app, grpc))
}

fn rest_router() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/healthz", get(healthz))
        .route("/api/healthz", get(healthz))
}

async fn grpc_router(config: &Config) -> Result<Router> {
    Ok(Router::new()
        .nest_tonic(
            tonic_reflection::server::Builder::configure()
                .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
                .build()
                .unwrap()
        )
        .nest_tonic(tonic_web::enable(MlSandboxServiceServer::new(MlSandboxServiceHandler::new(config).await?))))
}

struct MlSandboxServiceHandler {
    database: Database,
    queue: Queue,
}

impl MlSandboxServiceHandler {
    pub async fn new(config: &Config) -> Result<Self> {
        Ok(Self {
            database: Database::new(&config.get_string("database.node")?).await?,
            queue: Queue::new(&config.get_string("queue.node")?),
        })
    }
}

#[tonic::async_trait]
impl MlSandboxService for MlSandboxServiceHandler {
    async fn generate_image(&self, req: Request<GenerateImageRequest>) -> Result<Response<GenerateImageResponse>, Status> {
        let req = req.into_inner();

        let task_id = generate_task_id();
        self.queue.publish_task_message(&TaskMessage {
            id: task_id.clone(),
            prompt: req.prompt.clone(),
        }).await;
        self.database.new_task(&task_id, &req.prompt).await.unwrap();

        Ok(tonic::Response::new(GenerateImageResponse {
            id: task_id,
        }))
    }
}

fn generate_task_id() -> String {
    let mut rng = rand::thread_rng();
    Alphanumeric.sample_iter(&mut rng)
        .take(14)
        .map(char::from)
        .collect()
}

async fn root() -> &'static str {
    "sandbox"
}

async fn healthz() -> &'static str {
    "ok"
}


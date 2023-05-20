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
    rpc::{
        ml_sandbox_service_server::{MlSandboxService, MlSandboxServiceServer},
        FILE_DESCRIPTOR_SET,
        GenerateImageRequest,
        GenerateImageResponse,
        TaskId,
        TaskStatus,
        HistoryRequest,
        TaskHistory,
    },
    crate::state::{
        database::Database, 
        queue::{Queue, TaskMessage}, 
        storage::Storage,
    },
};

#[derive(Deserialize)]
struct TokenClaims {
    sub: String,
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
    storage: Storage,

    token_decoding_key: DecodingKey,
}

impl MlSandboxServiceHandler {
    pub async fn new(config: &Config) -> Result<Self> {
        Ok(Self {
            database: Database::new(&config.get_string("database.node")?).await?,
            queue: Queue::new(&config.get_string("queue.node")?),
            storage: Storage::new(config),
            token_decoding_key: DecodingKey::from_rsa_pem(&config.get_string("token.decoding_key")?.as_bytes()).unwrap(),
        })
    }

    pub fn decode_token(&self, token: &str) -> String {
        jsonwebtoken::decode::<TokenClaims>(token, &self.token_decoding_key, &Validation::new(Algorithm::RS384)).unwrap().claims.sub
    }
}

#[tonic::async_trait]
impl MlSandboxService for MlSandboxServiceHandler {
    async fn generate_image(&self, req: Request<GenerateImageRequest>) -> Result<Response<GenerateImageResponse>, Status> {
        let headers: http::HeaderMap = req.metadata().clone().into_headers();
        let user_id = headers.get("x-access-token")
            .map(|v| v.to_str().unwrap().to_owned())
            .map(|v| self.decode_token(&v));

        let req = req.into_inner();

        let task_id = generate_task_id();
        self.queue.publish_task_message(&TaskMessage {
            id: task_id.clone(),
            prompt: req.prompt.clone(),
        }).await;
        self.database.new_task(user_id, &task_id, &req.prompt).await.unwrap();

        Ok(tonic::Response::new(GenerateImageResponse {
            id: task_id,
        }))
    }

    async fn get_task_status(&self, req: Request<TaskId>) -> Result<Response<TaskStatus>, Status> {
        let task_id = req.into_inner();

        let task = self.database.get_task(&task_id.id).await;
        let is_complete = task.status == "complete";
        let image = if is_complete {
            Some(self.storage.get_generated_image(&task_id.id).await)
        } else {
            None
        };

        Ok(tonic::Response::new(TaskStatus {
            prompt: task.prompt,
            is_complete,
            image,
        }))
    }

    async fn get_task_history(&self, req: Request<HistoryRequest>) -> Result<Response<TaskHistory>, Status> {
        let headers = req.metadata().clone().into_headers();
        let user_id = match headers.get("x-access-token")
            .map(|v| v.to_str().unwrap().to_owned())
            .map(|v| self.decode_token(&v)) {
            Some(v) => v,
            None => return Err(Status::unauthenticated("unauthenticated")),
        };

        let tasks = self.database.get_user_tasks(&user_id).await;

        unimplemented!()
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


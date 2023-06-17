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
        ml_sandbox_service_server::{MlSandboxService, MlSandboxServiceServer},
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
        SubmitTaskResultRequest,
        SubmitTaskResultResponse,
    },
    crate::state::{
        database::{Database, Task}, 
        storage::Storage,
    },
};

#[derive(Deserialize)]
struct TokenClaims {
    sub: String,
}

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
        .add_service(MlSandboxServiceServer::new(MlSandboxServiceHandler::new(config).await.unwrap()))
        .serve(format!("0.0.0.0:{}", port).parse().unwrap())
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
    storage: Storage,

    token_decoding_key: DecodingKey,
    worker_token: String,
}

impl MlSandboxServiceHandler {
    pub async fn new(config: &Config) -> Result<Self> {
        Ok(Self {
            database: Database::new( &config.get_string("database.connection_string")?).await?,
            storage: Storage::new(config),
            token_decoding_key: DecodingKey::from_rsa_pem(&config.get_string("token.decoding_key")?.as_bytes()).unwrap(),
            worker_token: config.get_string("token.worker_token").unwrap(),
        })
    }

    pub fn decode_token(&self, token: &str) -> String {
        jsonwebtoken::decode::<TokenClaims>(token, &self.token_decoding_key, &Validation::new(Algorithm::RS384)).unwrap().claims.sub
    }

    pub async fn task_to_task_status(&self, task: Task) -> TaskStatus {
        let is_complete = task.status == "complete";
        let image = if is_complete {
            Some(self.storage.get_generated_image(&task.id).await)
        } else {
            None
        };

        TaskStatus {
            prompt: task.prompt,
            is_complete,
            image,
            id: task.id,
        }
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
        self.database.new_task(user_id, &task_id, &req.prompt).await.unwrap();

        Ok(tonic::Response::new(GenerateImageResponse {
            id: task_id,
        }))
    }

    async fn get_task_status(&self, req: Request<TaskId>) -> Result<Response<TaskStatus>, Status> {
        let task_id = req.into_inner();
        Ok(tonic::Response::new(self.task_to_task_status(self.database.get_task(&task_id.id).await).await))
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

        let mut result = Vec::new();
        for task in tasks.into_iter() {
            result.push(self.task_to_task_status(task).await);
        }

        Ok(tonic::Response::new(TaskHistory { tasks: result }))
    }

    async fn get_task_to_run(&self, req: Request<GetTaskToRunRequest>) -> Result<Response<GetTaskToRunResponse>, Status> {
        let headers = req.metadata().clone().into_headers();
        let token = match headers.get("x-access-token").map(|v| v.to_str().unwrap().to_owned()) {
            Some(v) => v,
            None => return Err(Status::unauthenticated("unauthenticated")),
        };

        if token != self.worker_token {
            return Err(Status::unauthenticated("wrong_token"));
        }

        let task_to_run = self.database.get_any_new_task().await;
        Ok(tonic::Response::new(GetTaskToRunResponse {
            task_to_run: task_to_run.map(|v| TaskToRun {
                id: v.id,
                prompt: v.prompt,
            })
        }))
    }

    async fn submit_task_result(&self, req: Request<SubmitTaskResultRequest>) -> Result<Response<SubmitTaskResultResponse>, Status> {
        let headers = req.metadata().clone().into_headers();
        let token = match headers.get("x-access-token").map(|v| v.to_str().unwrap().to_owned()) {
            Some(v) => v,
            None => return Err(Status::unauthenticated("unauthenticated")),
        };

        if token != self.worker_token {
            return Err(Status::unauthenticated("wrong_token"));
        }

        let req = req.into_inner();

        self.database.mark_task_as_complete(&req.id).await.unwrap();
        self.storage.save_generated_image(&req.id, &req.image).await;
        
        Ok(tonic::Response::new(SubmitTaskResultResponse {
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


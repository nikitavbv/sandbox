use {
    std::{io::Cursor, pin::Pin, future::Future, sync::Arc},
    tonic::{Status, Request, Response},
    tracing::{info, error},
    tokio::sync::Mutex,
    config::Config,
    rand::distributions::{Alphanumeric, Distribution},
    axum::{Router, routing::get},
    axum_tonic::{NestTonic, RestGrpcService},
    rpc::{
        ml_sandbox_service_server::{MlSandboxService, MlSandboxServiceServer},
        FILE_DESCRIPTOR_SET,
        InferenceRequest,
        InferenceResponse,
    },
    crate::{
        data::resolver::DataResolver,
        models::{
            io::ModelData,
            Model,
            ModelDefinition,
            image_generation::StableDiffusionImageGenerationModel,
            text_generation::TextGenerationModel,
            text_summarization::TextSummarizationModel,
        },
        scheduling::{
            registry::ModelRegistry,
            scheduler::Scheduler,
            simple::SimpleScheduler,
        },
        context::Context,
    },
};

pub async fn run_axum_server(config: &Config, scheduler: Box<dyn Scheduler + Send + Sync>) {
    let host = config.get_string("server.host").unwrap_or("0.0.0.0".to_owned());
    let port = config.get_int("server.port").unwrap_or(8080);
    let addr = format!("{}:{}", host, port).parse().unwrap();

    info!("starting axum server on {:?}", addr);
    
    axum::Server::bind(&addr)
        .serve(service(config, scheduler).await.into_make_service())
        .await
        .unwrap();
}

async fn service(config: &Config, scheduler: Box<dyn Scheduler + Send + Sync>) -> RestGrpcService {
    let app = rest_router();
    let grpc = grpc_router(config, scheduler).await;
    RestGrpcService::new(app, grpc)
}

fn rest_router() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/healthz", get(healthz))
}

async fn grpc_router(config: &Config, scheduler: Box<dyn Scheduler + Send + Sync>) -> Router {
    Router::new()
        .nest_tonic(
            tonic_reflection::server::Builder::configure()
                .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
                .build()
                .unwrap()
        )
        .nest_tonic(tonic_web::enable(MlSandboxServiceServer::new(MlSandboxServiceHandler::new(config, scheduler).await)))
}

struct MlSandboxServiceHandler {
    scheduler: Box<dyn Scheduler + Send + Sync>,
    context: Arc<Context>,
}

impl MlSandboxServiceHandler {
    pub async fn new(config: &Config, scheduler: Box<dyn Scheduler + Send + Sync>) -> Self {
        Self {
            scheduler,
            context: Arc::new(Context::new(DataResolver::new(config))),
        }
    }
}

#[tonic::async_trait]
impl MlSandboxService for MlSandboxServiceHandler {
    async fn run_model(&self, req: Request<InferenceRequest>) -> Result<Response<InferenceResponse>, Status> {
        let req = req.into_inner();

        let model_id = req.model.clone();
        let input = ModelData::from(req);
        let output = self.scheduler.run(self.context.clone(), &model_id, &input).await;
        
        if output.contains_key("image") {
            let key = &generate_output_data_key();
            self.context.data_resolver().put(format!("output/images/{}", key), &output.get_image("image").clone()).await;    
        }

        Ok(Response::new(InferenceResponse {
            entries: output.into(),
            worker: hostname::get().unwrap().to_string_lossy().to_string(),
        }))
    }

    async fn run_text_generation_model(&self, req: Request<InferenceRequest>) -> Result<Response<InferenceResponse>, Status> {
        let req = req.into_inner();
        let input = ModelData::from(req);
        let output = self.scheduler.run(self.context.clone(), "text-generation", &input).await;
        Ok(Response::new(InferenceResponse {
            entries: output.into(),
            worker: hostname::get().unwrap().to_string_lossy().to_string(),
        }))
    }
}

fn generate_output_data_key() -> String {
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

#[cfg(test)]
mod tests {
    use {
        http::StatusCode,
        axum_test_helper::TestClient,
        tracing_test::traced_test,
        super::*,
    };

    #[tokio::test]
    #[traced_test]
    async fn test_healthcheck() {
        let app = test_client().await;
        let res = app.get("/healthz").send().await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.text().await, "ok");
    }
    
    async fn test_client() -> TestClient {
        info!("creating test");
        TestClient::new(service(&test_config()).await)
    }
    
    fn test_config() -> Config {
        Config::builder()
            .add_source(config::File::with_name("../config.toml"))
            .add_source(config::Environment::with_prefix("SANDBOX"))
            .build()
            .unwrap()
    }
}

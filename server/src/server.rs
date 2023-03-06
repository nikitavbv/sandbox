use {
    std::{io::Cursor, pin::Pin, future::Future},
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
        RunSimpleModelRequest,
        RunSimpleModelResponse,
        TrainSimpleModelRequest,
        TrainSimpleModelResponse,
        InferenceRequest,
        InferenceResponse,
    },
    crate::{
        data::{
            object_storage::ObjectStorageDataResolver,
            file::FileDataResolver,
            cached_resolver::CachedResolver,
            resolver::DataResolver,
        },
        models::{
            io::ModelData,
            Model,
            ModelDefinition,
            SimpleMnistModel,
            image_generation::StableDiffusionImageGenerationModel,
            text_generation::TextGenerationModel,
            text_summarization::TextSummarizationModel,
        },
        scheduling::{
            registry::ModelRegistry,
            simple::SimpleScheduler,
        },
    },
};

pub async fn run_axum_server(config: &Config) {
    let host = config.get_string("server.host").unwrap_or("0.0.0.0".to_owned());
    let port = config.get_int("server.port").unwrap_or(8080);
    let addr = format!("{}:{}", host, port).parse().unwrap();

    info!("starting axum server on {:?}", addr);
    
    axum::Server::bind(&addr)
        .serve(service(config).await.into_make_service())
        .await
        .unwrap();
}

async fn service(config: &Config) -> RestGrpcService {
    let app = rest_router();
    let grpc = grpc_router(config).await;
    RestGrpcService::new(app, grpc)
}

fn rest_router() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/healthz", get(healthz))
}

async fn grpc_router(config: &Config) -> Router {
    Router::new()
        .nest_tonic(
            tonic_reflection::server::Builder::configure()
                .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
                .build()
                .unwrap()
        )
        .nest_tonic(tonic_web::enable(MlSandboxServiceServer::new(MlSandboxServiceHandler::new(config).await)))
}

struct MlSandboxServiceHandler {
    scheduer: SimpleScheduler,

    model: Mutex<Option<SimpleMnistModel>>,
    stable_diffusion: Mutex<Option<StableDiffusionImageGenerationModel>>,
    text_generation_model: Mutex<Option<TextGenerationModel>>,
    text_summarization_model: Mutex<Option<TextSummarizationModel>>,

    stable_diffusion_data_resolver: CachedResolver<ObjectStorageDataResolver, FileDataResolver>,

    output_storage: ObjectStorageDataResolver,
}

impl MlSandboxServiceHandler {
    pub async fn new(config: &Config) -> Self {
        let object_storage_resolver = ObjectStorageDataResolver::new_with_config(
            "nikitavbv-sandbox".to_owned(), 
            "data/models/stable-diffusion".to_owned(), 
            config
        );

        let file_resolver = FileDataResolver::new("./data/stable-diffusion".to_owned());
        let data_resolver = CachedResolver::new(object_storage_resolver, file_resolver);

        let output_storage = ObjectStorageDataResolver::new_with_config(
            "nikitavbv-sandbox".to_owned(),
            "output".to_owned(),
            config
        );

        let registry = ModelRegistry::new(config).await
            .with_definition(ModelDefinition::new("image_generation".to_owned(), image_generation_model_factory));

        Self {
            scheduer: SimpleScheduler::new(registry),
            
            model: Mutex::new(None),
            stable_diffusion: Mutex::new(None),
            text_generation_model: Mutex::new(None),
            text_summarization_model: Mutex::new(None),

            stable_diffusion_data_resolver: data_resolver,

            output_storage,
        }
    }
}

#[tonic::async_trait]
impl MlSandboxService for MlSandboxServiceHandler {
    async fn run_simple_model(&self, req: Request<RunSimpleModelRequest>) -> Result<Response<RunSimpleModelResponse>, Status> {
        let image = req.into_inner().image;
        let _image = image::io::Reader::new(Cursor::new(&image)).with_guessed_format().unwrap().decode().unwrap();

        let _model = self.model.lock().await;
        
        Ok(Response::new(RunSimpleModelResponse {}))
    }

    async fn train_simple_model(&self, _req: Request<TrainSimpleModelRequest>) -> Result<Response<TrainSimpleModelResponse>, Status> {
        let mut model = self.model.lock().await;
        if model.is_none() {
            *model = Some(SimpleMnistModel::new());
        }

        model.as_ref().unwrap().train();
        
        Ok(Response::new(TrainSimpleModelResponse {}))
    }

    async fn run_model(&self, req: Request<InferenceRequest>) -> Result<Response<InferenceResponse>, Status> {
        let req = req.into_inner();
        // let mut model = self.registry.get(&req.model);

        Ok(match req.model.as_str() {
            "image_generation" => {
                let mut model = self.stable_diffusion.lock().await;
                if model.is_none() {    
                    *model = Some(StableDiffusionImageGenerationModel::new(&self.stable_diffusion_data_resolver).await);
                }
                
                let input = ModelData::from(req);
                let output = model.as_ref().unwrap().run(&input);
                
                let key = &generate_output_data_key();
                self.output_storage.put(key, output.get_image("image").clone()).await;

                Response::new(InferenceResponse {
                    entries: output.into(),
                    worker: hostname::get().unwrap().to_string_lossy().to_string(),
                })
            },
            "text_generation" => {
                let mut model = self.text_generation_model.lock().await;
                if model.is_none() {
                    *model = Some(TextGenerationModel::new());
                }

                let input = ModelData::from(req);
                let output = model.as_ref().unwrap().run(&input);

                Response::new(InferenceResponse {
                    entries: output.into(),
                    worker: hostname::get().unwrap().to_string_lossy().to_string(),
                })
            },
            "text_summarization" => {
                let mut model = self.text_summarization_model.lock().await;
                if model.is_none() {
                    *model = Some(TextSummarizationModel::new());
                }

                let input = ModelData::from(req);
                let output = model.as_ref().unwrap().run(&input);

                Response::new(InferenceResponse {
                    entries: output.into(),
                    worker: hostname::get().unwrap().to_string_lossy().to_string(),
                })
            },
            other => {
                error!("unexpected model: {}", other);
                Response::new(InferenceResponse {
                    entries: ModelData::new().into(),
                    worker: hostname::get().unwrap().to_string_lossy().to_string(),
                })
            }
        })
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

fn image_generation_model_factory(resolver: FileDataResolver) -> Pin<Box<dyn Future<Output = Box<dyn Model + Send>>>> {
    unimplemented!()
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

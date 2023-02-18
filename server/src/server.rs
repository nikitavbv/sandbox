use {
    std::io::Cursor,
    tonic::{transport::Server, Status, Request, Response},
    tokio::sync::Mutex,
    config::Config,
    rpc::{
        ml_sandbox_service_server::{MlSandboxService, MlSandboxServiceServer},
        FILE_DESCRIPTOR_SET,
        RunSimpleModelRequest,
        RunSimpleModelResponse,
        TrainSimpleModelRequest,
        TrainSimpleModelResponse,
        RunImageGenerationModelRequest,
        RunImageGenerationModelResponse,
    },
    crate::{
        data::{
            object_storage::ObjectStorageDataResolver,
            file::FileDataResolver,
            cached_resolver::CachedResolver,
        },
        models::{
            SimpleMnistModel,
            image_generation::StableDiffusionImageGenerationModel,
        },
    },
};

pub async fn run_server(config: &Config) {
    Server::builder()
        .accept_http1(true)
        .add_service(
            tonic_reflection::server::Builder::configure()
                .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
                .build()
                .unwrap()
        )
        .add_service(tonic_web::enable(MlSandboxServiceServer::new(MlSandboxServiceHandler::new(config).await)))
        .serve("0.0.0.0:8080".parse().unwrap())
        .await
        .unwrap();
}

struct MlSandboxServiceHandler {
    model: Mutex<SimpleMnistModel>,
    stable_diffusion: Mutex<StableDiffusionImageGenerationModel>,
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

        Self {
            model: Mutex::new(SimpleMnistModel::new()),
            stable_diffusion: Mutex::new(StableDiffusionImageGenerationModel::new(data_resolver).await),
        }
    }
}

#[tonic::async_trait]
impl MlSandboxService for MlSandboxServiceHandler {
    async fn run_simple_model(&self, req: Request<RunSimpleModelRequest>) -> Result<Response<RunSimpleModelResponse>, Status> {
        let image = req.into_inner().image;
        let image = image::io::Reader::new(Cursor::new(&image)).with_guessed_format().unwrap().decode().unwrap();

        let model = self.model.lock().await;
        
        Ok(Response::new(RunSimpleModelResponse {}))
    }

    async fn train_simple_model(&self, req: Request<TrainSimpleModelRequest>) -> Result<Response<TrainSimpleModelResponse>, Status> {
        let model = self.model.lock().await;

        model.train();
        
        Ok(Response::new(TrainSimpleModelResponse {}))
    }

    async fn run_image_generation_model(&self, req: Request<RunImageGenerationModelRequest>) -> Result<Response<RunImageGenerationModelResponse>, Status> {
        let model = self.stable_diffusion.lock().await;
        let prompt = req.into_inner().prompt.clone();

        let image = model.run(&prompt);
        
        Ok(Response::new(RunImageGenerationModelResponse {
            image,
        }))
    }
}
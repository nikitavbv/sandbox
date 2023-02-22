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
        InferenceRequest,
        RunTextGenerationModelResponse,
    },
    crate::{
        data::{
            object_storage::ObjectStorageDataResolver,
            file::FileDataResolver,
            cached_resolver::CachedResolver,
        },
        models::{
            io::ModelData,
            SimpleMnistModel,
            image_generation::StableDiffusionImageGenerationModel,
            text_generation::TextGenerationModel,
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
    model: Mutex<Option<SimpleMnistModel>>,
    stable_diffusion: Mutex<Option<StableDiffusionImageGenerationModel>>,
    text_generation_model: Mutex<Option<TextGenerationModel>>,

    stable_diffusion_data_resolver: CachedResolver<ObjectStorageDataResolver, FileDataResolver>,
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
            model: Mutex::new(None),
            stable_diffusion: Mutex::new(None),
            text_generation_model: Mutex::new(None),

            stable_diffusion_data_resolver: data_resolver,
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

    async fn train_simple_model(&self, _req: Request<TrainSimpleModelRequest>) -> Result<Response<TrainSimpleModelResponse>, Status> {
        let mut model = self.model.lock().await;
        if model.is_none() {
            *model = Some(SimpleMnistModel::new());
        }

        model.as_ref().unwrap().train();
        
        Ok(Response::new(TrainSimpleModelResponse {}))
    }

    async fn run_image_generation_model(&self, req: Request<RunImageGenerationModelRequest>) -> Result<Response<RunImageGenerationModelResponse>, Status> {
        let mut model = self.stable_diffusion.lock().await;
        if model.is_none() {    
            *model = Some(StableDiffusionImageGenerationModel::new(&self.stable_diffusion_data_resolver).await);
        }
        
        let prompt = req.into_inner().prompt.clone();

        let input = ModelData::new().with_text("prompt".to_owned(), prompt.to_owned());
        let image = model.as_ref().unwrap().run(&input);
        
        Ok(Response::new(RunImageGenerationModelResponse {
            image,
        }))
    }

    async fn run_text_generation_model(&self, req: Request<InferenceRequest>) -> Result<Response<RunTextGenerationModelResponse>, Status> {
        let mut model = self.text_generation_model.lock().await;
        if model.is_none() {
            *model = Some(TextGenerationModel::new());
        }

        let input = ModelData::from(req.into_inner());
        let text = model.as_ref().unwrap().run(&input);

        Ok(Response::new(RunTextGenerationModelResponse {
            text,
        }))
    }
}
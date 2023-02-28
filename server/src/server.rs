use {
    std::io::Cursor,
    tonic::{transport::Server, Status, Request, Response},
    tracing::info,
    tokio::sync::Mutex,
    config::Config,
    rand::distributions::{Alphanumeric, Distribution},
    rpc::{
        ml_sandbox_service_server::{MlSandboxService, MlSandboxServiceServer},
        FILE_DESCRIPTOR_SET,
        RunSimpleModelRequest,
        RunSimpleModelResponse,
        TrainSimpleModelRequest,
        TrainSimpleModelResponse,
        RunImageGenerationModelResponse,
        InferenceRequest,
        RunTextGenerationModelResponse,
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
            SimpleMnistModel,
            image_generation::StableDiffusionImageGenerationModel,
            text_generation::TextGenerationModel,
        },
    },
};

pub async fn run_server(config: &Config) {
    let host = config.get_string("server.host").unwrap_or("0.0.0.0".to_owned());
    let port = config.get_int("server.port").unwrap_or(8080);
    let addr = format!("{}:{}", host, port).parse().unwrap();

    info!("starting server on {:?}", addr);

    Server::builder()
        .accept_http1(true)
        .add_service(
            tonic_reflection::server::Builder::configure()
                .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
                .build()
                .unwrap()
        )
        .add_service(tonic_web::enable(MlSandboxServiceServer::new(MlSandboxServiceHandler::new(config).await)))
        .serve(addr)
        .await
        .unwrap();
}

struct MlSandboxServiceHandler {
    model: Mutex<Option<SimpleMnistModel>>,
    stable_diffusion: Mutex<Option<StableDiffusionImageGenerationModel>>,
    text_generation_model: Mutex<Option<TextGenerationModel>>,

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

        Self {
            model: Mutex::new(None),
            stable_diffusion: Mutex::new(None),
            text_generation_model: Mutex::new(None),

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

    async fn run_image_generation_model(&self, req: Request<InferenceRequest>) -> Result<Response<RunImageGenerationModelResponse>, Status> {
        let mut model = self.stable_diffusion.lock().await;
        if model.is_none() {    
            *model = Some(StableDiffusionImageGenerationModel::new(&self.stable_diffusion_data_resolver).await);
        }
        
        let input = ModelData::from(req.into_inner());
        let image = model.as_ref().unwrap().run(&input);
        
        let key = &generate_output_data_key();
        self.output_storage.put(key, image.clone()).await;

        Ok(Response::new(RunImageGenerationModelResponse {
            image,
            worker: hostname::get().unwrap().to_string_lossy().to_string(),
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
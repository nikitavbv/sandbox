use {
    std::time::Duration,
    tracing::{info, warn},
    tokio::time::sleep,
    tonic::{
        codegen::InterceptedService,
        service::Interceptor,
        transport::ClientTlsConfig,
        Status,
    },
    sandbox_common::{
        utils::{init_logging, load_config},
        messages::TaskMessage,
    },
    rpc::{
        ml_sandbox_service_client::MlSandboxServiceClient,
        GetTaskToRunRequest,
        SubmitTaskResultRequest,
    },
    crate::{
        model::StableDiffusionImageGenerationModel,
        storage::Storage,
    },
};

pub mod model;
pub mod storage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();
    let config = load_config();
    
    info!("sandbox worker started");
    let mut client = MlSandboxServiceClient::with_interceptor(
        tonic::transport::Channel::from_static("https://sandbox.nikitavbv.com/api")
            .tls_config(ClientTlsConfig::new())
            .unwrap()
            .connect()
            .await
            .unwrap(),
        AuthTokenSetterInterceptor::new()
    );
    //let storage = Storage::new(&config);

    /*info!("loading model");
    let model = StableDiffusionImageGenerationModel::new(&storage).await;
    info!("model loaded");


    info!("generating image for prompt: {}", payload.prompt);
    let image = model.run(&payload.prompt);
    info!("finished generating image");
    storage.save_generated_image(&payload.id, &image).await;
    database.mark_task_as_complete(&payload.id).await.unwrap();

    if let Err(err) = consumer.commit_message(&v, CommitMode::Async) {
        warn!("failed to commit offsets: {:?}", err);
    }*/

    Ok(())
}

pub struct AuthTokenSetterInterceptor {
}

impl AuthTokenSetterInterceptor {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl Interceptor for AuthTokenSetterInterceptor {
    fn call(&mut self, mut req: Request<()>) -> Result<Request<()>, Status> {
        let auth_header_value = MetadataValue::try_from(format!("Bearer token")).expect("failed to create metadata");
        req.metadata_mut().insert("authorization", auth_header_value);
        Ok(req)
    }
}


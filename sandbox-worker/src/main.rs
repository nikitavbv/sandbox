use {
    std::time::Duration,
    tracing::{info, warn},
    tokio::time::sleep,
    tonic::{
        codegen::InterceptedService,
        service::Interceptor,
        transport::ClientTlsConfig,
        metadata::MetadataValue,
        Status,
        Request,
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
        tonic::transport::Channel::from_static("http://localhost:8081")
            //.tls_config(ClientTlsConfig::new())
            //.unwrap()
            .connect()
            .await
            .unwrap(),
        AuthTokenSetterInterceptor::new(config.get_string("token.worker_token").unwrap()),
    );
    let res = client.get_task_to_run(GetTaskToRunRequest {}).await.unwrap();
    info!("res: {:?}", res);
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
    token: String,
}

impl AuthTokenSetterInterceptor {
    pub fn new(token: String) -> Self {
        Self {
            token,
        }
    }
}

impl Interceptor for AuthTokenSetterInterceptor {
    fn call(&mut self, mut req: Request<()>) -> Result<Request<()>, Status> {
        let auth_header_value: MetadataValue<tonic::metadata::Ascii> = MetadataValue::try_from(&self.token).expect("failed to create metadata");
        req.metadata_mut().insert("x-access-token", auth_header_value);
        Ok(req)
    }
}


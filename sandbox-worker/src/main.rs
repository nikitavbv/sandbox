use {
    std::time::Duration,
    tracing::{info, warn},
    tokio::time::sleep,
    tonic::{
        service::Interceptor,
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
    let endpoint = config.get_string("worker.endpoint").unwrap();
    let mut client = MlSandboxServiceClient::with_interceptor(
        tonic::transport::Channel::from_shared(endpoint)
            .unwrap()
            .connect()
            .await
            .unwrap(),
        AuthTokenSetterInterceptor::new(config.get_string("token.worker_token").unwrap()),
    );
    
    let storage = Storage::new(&config);

    info!("loading model");
    let model = StableDiffusionImageGenerationModel::new(&storage).await;
    info!("model loaded");

    loop {
        let res = client.get_task_to_run(GetTaskToRunRequest {}).await.unwrap().into_inner();
    
        let task = match res.task_to_run {
            Some(v) => v,
            None => {
                info!("no tasks at this moment, waiting...");
                sleep(Duration::from_secs(10)).await;
                continue;
            }
        };

        let prompt = task.prompt;
        let id = task.id;
        info!("generating image for prompt: {}, task id: {}", prompt, id);
        let image = model.run(&prompt);
        info!("finished generating image");
        
        client.submit_task_result(SubmitTaskResultRequest {
            id,
            image,
        }).await.unwrap();

        info!("finished processing task");
    }
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


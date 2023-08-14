use {
    std::{time::Duration, sync::Arc},
    tracing::{info, error},
    tokio::{time::sleep, sync::Mutex},
    tonic::{
        service::Interceptor,
        metadata::MetadataValue,
        Status,
        Request,
        transport::Channel,
        codegen::InterceptedService,
    },
    sandbox_common::utils::{init_logging, load_config},
    rpc::{
        self,
        sandbox_service_client::SandboxServiceClient,
        GetTaskToRunRequest,
        UpdateTaskStatusRequest,
        CreateTaskAssetRequest,
        task_params::{Params, ImageGenerationParams, ChatMessageGenerationParams},
        TaskId,
    },
    crate::{
        stable_diffusion::{StableDiffusionImageGenerationModel, ImageGenerationStatus},
        llama::{LlamaChatModel, Message, Role},
        storage::Storage,
    },
};

pub mod stable_diffusion;

pub mod llama;
pub mod storage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();
    let config = load_config();
    
    info!("sandbox worker started");
    let endpoint = config.get_string("worker.endpoint").unwrap();
    let client = Arc::new(Mutex::new(SandboxServiceClient::with_interceptor(
        Channel::from_shared(endpoint)
            .unwrap()
            .connect()
            .await
            .unwrap(),
        AuthTokenSetterInterceptor::new(config.get_string("token.worker_token").unwrap()),
    )));
    
    let storage = Storage::new(&config);

    info!("loading models");
    let text_to_image_model = StableDiffusionImageGenerationModel::new(&storage).await; 
    info!("text to image model loaded");
    let chat_model = LlamaChatModel::new(&storage).await;
    info!("chat model loaded");

    loop {
        let res = match client.lock().await.get_task_to_run(GetTaskToRunRequest {}).await {
            Ok(v) => v.into_inner(),
            Err(err) => {
                error!("failed to request task to run: {:?}", err);
                sleep(Duration::from_secs(10)).await;
                continue;
            }
        };
    
        let task = match res.task_to_run {
            Some(v) => v,
            None => {
                info!("no tasks at this moment, waiting...");
                sleep(Duration::from_secs(10)).await;
                continue;
            }
        };

        match task.params.unwrap().params.unwrap() {
            Params::ImageGeneration(image_generation) => run_image_generation_task(client.clone(), &text_to_image_model, task.id.unwrap(), &image_generation).await,
            Params::ChatMessageGeneration(chat_message_generation) => run_chat_message_generation_task(client.clone(), &chat_model, task.id.unwrap(), &chat_message_generation).await,
        };

        info!("finished processing task");
    }
}

async fn run_image_generation_task(
    client: Arc<Mutex<SandboxServiceClient<InterceptedService<Channel, AuthTokenSetterInterceptor>>>>,
    text_to_image_model: &StableDiffusionImageGenerationModel,
    id: TaskId, 
    params: &ImageGenerationParams
) {
    let prompt = params.prompt.clone();
    let total_images = params.number_of_images;

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    {
        let id = id.clone();
        let client = client.clone();

        tokio::spawn(async move {
            let mut current_image = 0;

            while let Some(update) = rx.recv().await {
                match update {
                    ImageGenerationStatus::Finished => break,
                    ImageGenerationStatus::StartedImageGeneration { current_image: i } => {
                        current_image = i;
                    }
                    ImageGenerationStatus::InProgress { current_step, total_steps } => {
                        let res = client.lock().await.update_task_status(UpdateTaskStatusRequest {
                            id: Some(id.clone()),
                            task_status: Some(rpc::update_task_status_request::TaskStatus::InProgress(rpc::InProgressTaskDetails {
                                current_step,
                                total_steps,
                                current_image,
                            })),
                        }).await;

                        if let Err(err) = res {
                            error!("failed to report task status: {:?}", err);
                        }
                    },
                }
            }
        });
    }

    for image in 0..total_images {
        tx.send(ImageGenerationStatus::StartedImageGeneration { current_image: image }).unwrap();
        info!("generating image ({}/{}) for prompt: {}, task id: {}", image + 1, total_images, prompt, id.id);

        let image = text_to_image_model.run(&prompt, tx.clone());
        info!("finished generating image");
        
        client.lock().await.create_task_asset(CreateTaskAssetRequest {
            task_id: Some(id.clone()),
            image,
        }).await.unwrap();   
    }

    tx.send(ImageGenerationStatus::Finished).unwrap();
    client.lock().await.update_task_status(UpdateTaskStatusRequest {
        id: Some(id.clone()),
        task_status: Some(rpc::update_task_status_request::TaskStatus::Finished(rpc::FinishedTaskDetails {})),
    }).await.unwrap();
}

async fn run_chat_message_generation_task(
    client: Arc<Mutex<SandboxServiceClient<InterceptedService<Channel, AuthTokenSetterInterceptor>>>>,
    chat_model: &LlamaChatModel,
    id: TaskId,
    params: &ChatMessageGenerationParams
) {
    let res = chat_model.chat(vec![
        Message::new(Role::User, "Hi!".to_owned()),
    ]);

    info!("finished running chat message generation: {:?}", res);

    client.lock().await.update_task_status(UpdateTaskStatusRequest {
        id: Some(id.clone()),
        task_status: Some(rpc::update_task_status_request::TaskStatus::Finished(rpc::FinishedTaskDetails {})),
    }).await.unwrap();
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


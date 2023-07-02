use {
    tonic::{Status, Request, Response},
    serde::Deserialize,
    anyhow::Result,
    config::Config,
    jsonwebtoken::{DecodingKey, Validation, Algorithm},
    rand::distributions::{Alphanumeric, Distribution},
    rpc::{
        sandbox_service_server::{SandboxService, SandboxServiceServer},
        FILE_DESCRIPTOR_SET,
        GenerateImageRequest,
        GenerateImageResponse,
        TaskId as RpcTaskId,
        TaskStatus,
        HistoryRequest,
        TaskHistory,
        GetTaskToRunRequest,
        GetTaskToRunResponse,
        TaskToRun,
        UpdateTaskStatusRequest,
        UpdateTaskStatusResponse,
        Status as RpcStatus,
    },
    crate::{
        entities::TaskId,
        state::{
            database::Database,
            storage::Storage,
        },
        services::update_task_status,
    },
};

#[derive(Deserialize)]
struct TokenClaims {
    sub: String,
}

pub struct SandboxServiceHandler {
    database: Database,
    storage: Storage,

    token_decoding_key: DecodingKey,
    worker_token: String,
}

impl SandboxServiceHandler {
    pub async fn new(config: &Config) -> Result<Self> {
        Ok(Self {
            database: Database::new(config, &config.get_string("database.connection_string")?).await?,
            storage: Storage::new(config),
            token_decoding_key: DecodingKey::from_rsa_pem(&config.get_string("token.decoding_key")?.as_bytes()).unwrap(),
            worker_token: config.get_string("token.worker_token").unwrap(),
        })
    }

    pub fn decode_token(&self, token: &str) -> String {
        jsonwebtoken::decode::<TokenClaims>(token, &self.token_decoding_key, &Validation::new(Algorithm::RS384)).unwrap().claims.sub
    }

    /*pub async fn task_to_task_status(&self, task: Task) -> TaskStatus {
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
    }*/
}

#[tonic::async_trait]
impl SandboxService for SandboxServiceHandler {
    async fn generate_image(&self, req: Request<GenerateImageRequest>) -> Result<Response<GenerateImageResponse>, Status> {
        /*let headers: http::HeaderMap = req.metadata().clone().into_headers();
        let user_id = headers.get("x-access-token")
            .map(|v| v.to_str().unwrap().to_owned())
            .map(|v| self.decode_token(&v));

        let req = req.into_inner();

        let task_id = generate_task_id();
        self.database.new_task(user_id, &task_id, &req.prompt).await.unwrap();

        Ok(tonic::Response::new(GenerateImageResponse {
            id: task_id,
        }))*/
        unimplemented!()
    }

    async fn get_task_status(&self, req: Request<RpcTaskId>) -> Result<Response<TaskStatus>, Status> {
        /*let task_id = req.into_inner();
        Ok(tonic::Response::new(self.task_to_task_status(self.database.get_task(&task_id.id).await).await))*/
        unimplemented!()
    }

    async fn get_task_history(&self, req: Request<HistoryRequest>) -> Result<Response<TaskHistory>, Status> {
        /*let headers = req.metadata().clone().into_headers();
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

        Ok(tonic::Response::new(TaskHistory { tasks: result }))*/
        unimplemented!()
    }

    async fn get_task_to_run(&self, req: Request<GetTaskToRunRequest>) -> Result<Response<GetTaskToRunResponse>, Status> {
        /*let headers = req.metadata().clone().into_headers();
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
        }))*/
        unimplemented!()
    }

    async fn update_task_status(&self, req: Request<UpdateTaskStatusRequest>) -> Result<Response<UpdateTaskStatusResponse>, Status> {
        let token = match extract_access_token(&req) {
            Some(v) => v,
            None => return Err(Status::unauthenticated("unauthenticated")),
        };

        if token != self.worker_token {
            return Err(Status::unauthenticated("wrong_token"));
        }

        let req = req.into_inner();
        let task_status = unimplemented!();

        update_task_status(&self.database, TaskId::new(req.id), task_status).await;

        Ok(Response::new(UpdateTaskStatusResponse {}))
    }
}

fn extract_access_token<T>(req: &Request<T>) -> Option<String> {
    let headers = req.metadata().clone().into_headers();
    headers.get("x-access-token").map(|v| v.to_str().unwrap().to_owned())
}

/*fn generate_task_id() -> String {
    let mut rng = rand::thread_rng();
    Alphanumeric.sample_iter(&mut rng)
        .take(14)
        .map(char::from)
        .collect()
}*/

use {
    tonic::{Status, Request, Response},
    serde::Deserialize,
    anyhow::Result,
    config::Config,
    jsonwebtoken::{DecodingKey, Validation, Algorithm},
    rand::distributions::{Alphanumeric, Distribution},
    rpc::{
        sandbox_service_server::{SandboxService, SandboxServiceServer},
        GetTaskToRunRequest,
        GetTaskToRunResponse,
        UpdateTaskStatusRequest,
        UpdateTaskStatusResponse,
        CreateTaskRequest,
        CreateTaskResponse,
        GetTaskRequest,
        GetTaskResponse,
        GetAllTasksRequest,
        GetAllTasksResponse,
    },
    crate::{
        entities::{TaskId, TaskStatus},
        state::{
            database::Database,
        },
    },
};

#[derive(Deserialize)]
struct TokenClaims {
    sub: String,
}

pub struct SandboxServiceHandler {
    database: Database,

    token_decoding_key: DecodingKey,
    worker_token: String,
}

impl SandboxServiceHandler {
    pub async fn new(config: &Config) -> Result<Self> {
        Ok(Self {
            database: Database::new(config, &config.get_string("database.connection_string")?).await?,
            token_decoding_key: DecodingKey::from_rsa_pem(&config.get_string("token.decoding_key")?.as_bytes()).unwrap(),
            worker_token: config.get_string("token.worker_token").unwrap(),
        })
    }

    pub fn decode_token(&self, token: &str) -> String {
        jsonwebtoken::decode::<TokenClaims>(token, &self.token_decoding_key, &Validation::new(Algorithm::RS384)).unwrap().claims.sub
    }
}

#[tonic::async_trait]
impl SandboxService for SandboxServiceHandler {
    async fn create_task(&self, req: Request<CreateTaskRequest>) -> Result<Response<CreateTaskResponse>, Status> {
        let headers: http::HeaderMap = req.metadata().clone().into_headers();
        let user_id = headers.get("x-access-token")
            .map(|v| v.to_str().unwrap().to_owned())
            .map(|v| self.decode_token(&v));

        let req = req.into_inner();

        let task_id = generate_task_id();
        self.database.new_task(user_id, &task_id, &req.prompt).await;

        Ok(Response::new(CreateTaskResponse {
            id: Some(rpc::TaskId::from(task_id)),
        }))
    }

    async fn get_task(&self, req: Request<GetTaskRequest>) -> Result<Response<GetTaskResponse>, Status> {
        let task_id = TaskId::from(req.into_inner().id.unwrap());
        let task = self.database.get_task(&task_id).await;

        Ok(Response::new(GetTaskResponse {
            task: Some(rpc::Task {
                prompt: task.prompt,
                status: match task.status {
                    TaskStatus::Pending => None,
                    TaskStatus::InProgress { current_step, total_steps } => Some(rpc::task::Status::InProgressDetails(rpc::InProgressTaskDetails {
                        current_step,
                        total_steps,
                    })),
                    TaskStatus::Finished { image } => Some(rpc::task::Status::FinishedDetails(rpc::FinishedTaskDetails {
                        image,
                    })),
                },
            }),
        }))
    }

    async fn get_all_tasks(&self, req: Request<GetAllTasksRequest>) -> Result<Response<GetAllTasksResponse>, Status> {
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
        let headers = req.metadata().clone().into_headers();
        let token = match headers.get("x-access-token").map(|v| v.to_str().unwrap().to_owned()) {
            Some(v) => v,
            None => return Err(Status::unauthenticated("unauthenticated")),
        };

        if token != self.worker_token {
            return Err(Status::unauthenticated("wrong_token"));
        }

        let task_to_run = self.database.get_any_new_task().await;

        Ok(Response::new(GetTaskToRunResponse {
            task_to_run: task_to_run.map(|v| rpc::get_task_to_run_response::TaskToRun {
                id: Some(rpc::TaskId::from(v.id)),
                prompt: v.prompt,
            }),
        }))
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
        let task_status = match req.task_status.unwrap() {
            rpc::update_task_status_request::TaskStatus::InProgress(in_progress) => TaskStatus::InProgress { current_step: in_progress.current_step, total_steps: in_progress.total_steps },
            rpc::update_task_status_request::TaskStatus::Finished(finished) => TaskStatus::Finished { image: finished.image },
        };

        let task_id = TaskId::from(req.id.unwrap());

        self.database.save_task_status(&task_id, &task_status).await;

        Ok(Response::new(UpdateTaskStatusResponse {}))
    }
}

fn extract_access_token<T>(req: &Request<T>) -> Option<String> {
    let headers = req.metadata().clone().into_headers();
    headers.get("x-access-token").map(|v| v.to_str().unwrap().to_owned())
}

fn generate_task_id() -> TaskId {
    let mut rng = rand::thread_rng();
    TaskId::new(Alphanumeric.sample_iter(&mut rng)
        .take(14)
        .map(char::from)
        .collect())
}
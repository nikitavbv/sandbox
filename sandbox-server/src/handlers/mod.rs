use {
    std::sync::Arc,
    tracing::{info, error},
    tonic::{Status, Request, Response},
    serde::{Serialize, Deserialize},
    anyhow::Result,
    chrono::{Utc, Timelike},
    jsonwebtoken::{EncodingKey, DecodingKey, Validation, Algorithm, errors::ErrorKind as JwtErrorKind},
    rand::distributions::{Alphanumeric, Distribution},
    prost_types::Timestamp,
    rpc::{
        self,
        sandbox_service_server::SandboxService,
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
        OAuthLoginRequest,
        OAuthLoginResponse,
        CreateTaskAssetRequest,
        CreateTaskAssetResponse,
        GetChatMessagesRequest,
        GetChatMessagesResponse,
        AddChatAssistantMessageRequest,
        AddChatAssistantMessageResponse,
    },
    crate::{
        entities::{Task, TaskId, TaskStatus, UserId, AssetId, TaskParams, ChatMessageRole},
        state::database::Database,
    },
};

pub mod rest;

#[derive(Serialize, Deserialize)]
struct TokenClaims {
    exp: usize,
    sub: String,
    email: String,
    name: String,
}

#[derive(Deserialize)]
struct OAuthCodeExchangeResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct UserInfoResponse {
    #[serde(rename = "id")]
    _id: String,
    email: String,
    name: String,
}

enum TokenDecodeResult {
    Token(String),
    TokenExpired,
    DecodeError(String),
}

pub struct SandboxServiceHandler {
    database: Arc<Database>,

    token_encoding_key: EncodingKey,
    token_decoding_key: DecodingKey,
    worker_token: String,
    oauth_secret: String,
}

impl SandboxServiceHandler {
    pub async fn new(database: Arc<Database>, token_encoding_key: EncodingKey, token_decoding_key: DecodingKey, worker_token: String, oauth_secret: String) -> Result<Self> {
        Ok(Self {
            database,
            token_encoding_key,
            token_decoding_key,
            worker_token,
            oauth_secret,
        })
    }

    fn issue_token(&self, id: &UserId, email: &str, name: &str) -> String {
        jsonwebtoken::encode(
            &jsonwebtoken::Header::new(Algorithm::RS384),
            &TokenClaims {
                exp: (Utc::now().timestamp() as usize) + (7 * 24 * 60 * 60),
                sub: id.to_string(),
                email: email.to_owned(),
                name: name.to_owned(),
            },
            &self.token_encoding_key,
        ).unwrap()
    }

    fn decode_token(&self, token: &str) -> TokenDecodeResult {
        match jsonwebtoken::decode::<TokenClaims>(token, &self.token_decoding_key, &Validation::new(Algorithm::RS384)) {
            Ok(v) => TokenDecodeResult::Token(v.claims.sub),
            Err(err) => match err.kind() {
                JwtErrorKind::ExpiredSignature => TokenDecodeResult::TokenExpired,
                other => TokenDecodeResult::DecodeError(format!("{:?}", other)),   
            }
        }
    }

    fn task_to_rpc_task(&self, task: Task, assets: Vec<AssetId>) -> rpc::Task {
        rpc::Task {
            id: Some(rpc::TaskId::from(task.id)),
            created_at: Some(Timestamp {
                seconds: task.created_at.timestamp(),
                nanos: task.created_at.nanosecond() as i32,
            }),
            status: match task.status {
                TaskStatus::Pending => Some(rpc::task::Status::PendingDetails(rpc::PendingTaskDetails {})),
                TaskStatus::InProgress { current_step, total_steps, current_image } => Some(rpc::task::Status::InProgressDetails(rpc::InProgressTaskDetails {
                    current_step,
                    total_steps,
                    current_image,
                })),
                TaskStatus::Finished => Some(rpc::task::Status::FinishedDetails(rpc::FinishedTaskDetails {})),
            },
            assets: assets.into_iter().map(|v| rpc::TaskAsset {
                id: v.to_string(),
            }).collect(),
            params: Some(rpc::TaskParams {
                params: Some(rpc::task_params::Params::from(task.params)),
            }),
        }
    }
}

#[tonic::async_trait]
impl SandboxService for SandboxServiceHandler {
    async fn o_auth_login(&self, req: Request<OAuthLoginRequest>) -> Result<Response<OAuthLoginResponse>, Status> {
        let req = req.into_inner();
        
        let client = reqwest::Client::new();
        let res = client
            .post("https://oauth2.googleapis.com/token")
            .form(&[
                ("client_id", "916750455653-biu6q4c7llj7q1k14h3qaquktcdlkeo4.apps.googleusercontent.com"),
                ("client_secret", &self.oauth_secret),
                ("code", &req.code),
                ("grant_type", "authorization_code"),
                ("redirect_uri", &req.redirect_uri),
            ])
            .send()
            .await;

        let res = match res {
            Ok(v) => v,
            Err(err) => {
                error!("failed to run code exchange request: {:?}", err);
                return Err(Status::internal("something went wrong"));
            }
        }.text().await.unwrap();

        let res: OAuthCodeExchangeResponse = match serde_json::from_str(&res) {
            Ok(v) => v,
            Err(err) => {
                error!("failed to get code exchange response: {:?} for response {:?}", err, res);
                return Err(Status::internal("failed to get token exchange response"));
            }
        };

        let res = client
            .get("https://www.googleapis.com/oauth2/v1/userinfo")
            .bearer_auth(res.access_token)
            .send()
            .await;

        let res = match res {
            Ok(v) => v,
            Err(err) => {
                error!("failed to request user info: {:?}", err);
                return Err(Status::internal("failed to request user info"));
            }
        };

        let res: UserInfoResponse = match res.json().await {
            Ok(v) => v,
            Err(err) => {
                error!("failed to get user info response: {:?}", err);
                return Err(Status::internal("failed to get user info response"));
            }
        };

        let user_id = self.database.create_or_get_user_by_email(&res.email).await;

        let token = self.issue_token(&user_id, &res.email, &res.name);
        info!("issued token for {}", res.email);

        Ok(Response::new(OAuthLoginResponse { token }))
    }

    async fn create_task(&self, req: Request<CreateTaskRequest>) -> Result<Response<CreateTaskResponse>, Status> {
        let headers: http::HeaderMap = req.metadata().clone().into_headers();
        let user_id = headers.get("x-access-token")
            .map(|v| v.to_str().unwrap().to_owned())
            .map(|v| self.decode_token(&v));

        let user_id = match user_id {
            Some(v) => match v {
                TokenDecodeResult::Token(t) => Some(t),
                TokenDecodeResult::DecodeError(err) => {
                    error!("error while decoding token: {:?}", err);
                    return Err(Status::internal("internal server error"));
                },
                TokenDecodeResult::TokenExpired => return Err(Status::unauthenticated("token expired")),
            },
            None => None,
        };

        let req = req.into_inner();

        let task_id = generate_task_id();
        let params = match req.params.unwrap().params.unwrap() {
            rpc::task_params::Params::ImageGeneration(v) => TaskParams::ImageGenerationParams {
                prompt: v.prompt,
                iterations: v.iterations,
                number_of_images: v.number_of_images,
            },
            rpc::task_params::Params::ChatMessageGeneration(_) => TaskParams::ChatMessageGenerationParams {
            },
        };

        self.database.new_task(user_id, &task_id, &params).await;

        Ok(Response::new(CreateTaskResponse {
            id: Some(rpc::TaskId::from(task_id)),
        }))
    }

    async fn get_task(&self, req: Request<GetTaskRequest>) -> Result<Response<GetTaskResponse>, Status> {
        let task_id = TaskId::from(req.into_inner().id.unwrap());
        let task = self.database.get_task(&task_id).await;
        let assets = self.database.get_task_assets(&task_id).await;

        Ok(Response::new(GetTaskResponse {
            task: Some(self.task_to_rpc_task(task, assets)),
        }))
    }

    async fn get_all_tasks(&self, req: Request<GetAllTasksRequest>) -> Result<Response<GetAllTasksResponse>, Status> {
        let headers = req.metadata().clone().into_headers();
        let user_id = match headers.get("x-access-token")
            .map(|v| v.to_str().unwrap().to_owned())
            .map(|v| self.decode_token(&v)) {
            Some(v) => match v {
                TokenDecodeResult::Token(t) => t,
                TokenDecodeResult::DecodeError(err) => {
                    error!("error while decoding token: {:?}", err);
                    return Err(Status::internal("internal server error"));
                },
                TokenDecodeResult::TokenExpired => return Err(Status::unauthenticated("token expired")),
            },
            None => return Err(Status::unauthenticated("unauthenticated")),
        };

        let tasks = self.database.get_user_tasks(&user_id).await;
        let mut rpc_tasks = Vec::new();

        for task in tasks {
            let assets = self.database.get_task_assets(&task.id).await;
            rpc_tasks.push(self.task_to_rpc_task(task, assets));
        }
        
        Ok(Response::new(GetAllTasksResponse { tasks: rpc_tasks }))
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

        self.database.update_worker_last_ping_time().await;

        let task_to_run = self.database.get_any_new_task().await;

        Ok(Response::new(GetTaskToRunResponse {
            task_to_run: task_to_run.map(|v| rpc::get_task_to_run_response::TaskToRun {
                id: Some(rpc::TaskId::from(v.id)),
                params: Some(rpc::TaskParams {
                    params: Some(rpc::task_params::Params::from(v.params)),
                }),
            }),
        }))
    }

    async fn create_task_asset(&self, req: Request<CreateTaskAssetRequest>) -> Result<Response<CreateTaskAssetResponse>, Status> {
        let token = match extract_access_token(&req) {
            Some(v) => v,
            None => return Err(Status::unauthenticated("unauthenticated")),
        };

        if token != self.worker_token {
            return Err(Status::unauthenticated("wrong_token"));
        }

        let req = req.into_inner();
        let task_id = TaskId::from(req.task_id.unwrap());

        self.database.create_task_asset(&task_id, req.image).await;

        Ok(Response::new(CreateTaskAssetResponse {}))
    }

    async fn get_chat_messages(&self, req: Request<GetChatMessagesRequest>) -> Result<Response<GetChatMessagesResponse>, Status> {
        let token = match extract_access_token(&req) {
            Some(v) => v,
            None => return Err(Status::unauthenticated("unauthenticated")),
        };

        if token != self.worker_token {
            return Err(Status::unauthenticated("wrong_token"));
        }

        let req = req.into_inner();
        let task_id = TaskId::from(req.task_id.unwrap());

        let messages = self.database.get_chat_messages(&task_id).await
            .into_iter()
            .map(|v| rpc::get_chat_messages_response::ChatMessage {
                message_id: Some(rpc::MessageId::from(v.message_id)),
                content: v.content,
                role: rpc::ChatMessageRole::from(v.role).into(),
                message_index: v.index,
            })
            .collect();

        Ok(Response::new(GetChatMessagesResponse {
            messages,
        }))
    }

    async fn add_chat_assistant_message(&self, req: Request<AddChatAssistantMessageRequest>) -> Result<Response<AddChatAssistantMessageResponse>, Status> {
        let token = match extract_access_token(&req) {
            Some(v) => v,
            None => return Err(Status::unauthenticated("unauthenticated")),
        };

        if token != self.worker_token {
            return Err(Status::unauthenticated("wrong_token"));
        }
        
        let req = req.into_inner();
        let task_id = TaskId::from(req.task_id.unwrap());

        self.database.append_chat_message(&task_id, req.content, ChatMessageRole::Assistant).await;

        Ok(Response::new(AddChatAssistantMessageResponse {}))
    }

    async fn update_task_status(&self, req: Request<UpdateTaskStatusRequest>) -> Result<Response<UpdateTaskStatusResponse>, Status> {
        let token = match extract_access_token(&req) {
            Some(v) => v,
            None => return Err(Status::unauthenticated("unauthenticated")),
        };

        if token != self.worker_token {
            return Err(Status::unauthenticated("wrong_token"));
        }

        self.database.update_worker_last_ping_time().await;

        let req = req.into_inner();
        let task_status = match req.task_status.unwrap() {
            rpc::update_task_status_request::TaskStatus::InProgress(in_progress) => TaskStatus::InProgress { 
                current_image: in_progress.current_image,
                current_step: in_progress.current_step, 
                total_steps: in_progress.total_steps,
            },
            rpc::update_task_status_request::TaskStatus::Finished(_) => TaskStatus::Finished,
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
use {
    std::time::Duration,
    anyhow::Result,
    sqlx::{postgres::PgPoolOptions,types::time::OffsetDateTime},
    config::Config,
    serde::{Serialize, Deserialize},
    s3::{Bucket, creds::Credentials, region::Region, error::S3Error},
    ulid::Ulid,
    chrono::{NaiveDateTime, DateTime, Utc},
    crate::entities::{
        TaskId, 
        TaskStatus, 
        Task, 
        UserId, 
        AssetId, 
        TaskParams, 
        ChatMessage, 
        MessageId,
        ChatMessageRole,
    },
};

struct PersistedTask {
    id: String,
    status: sqlx::types::JsonValue,
    created_at: OffsetDateTime,
    params: Option<sqlx::types::JsonValue>,
}

#[derive(Serialize, Deserialize, Debug)]
enum PersistedTaskStatus {
    Pending,
    InProgress {
        current_step: u32,
        total_steps: u32,
        current_image: Option<u32>,
    },
    Finished,
}

struct PersistedUserId {
    id: String,
}

struct PersistedAssetId {
    id: String,
}

#[derive(Serialize, Deserialize)]
struct PersistedTaskParams {
    iterations: u32,
    number_of_images: u32,
    prompt: Option<String>,
}

#[derive(Serialize, Deserialize)]
enum PersistedTaskParamsV2 {
    ImageGeneration {
        iterations: u32,
        number_of_images: u32,
        prompt: String,
    },
}

struct PersistedChatMessage {
    task_id: String,
    message_id: String,
    content: String,
    message_role: PersistedChatMessageRole,
    message_index: i32,
}

#[derive(sqlx::Type)]
#[sqlx(type_name = "message_role")]
enum PersistedChatMessageRole {
    System,
    User,
    Assistant,
}

pub struct Database {
    pool: sqlx::postgres::PgPool,
    bucket: s3::Bucket,
}

impl Database {
    pub async fn new(config: &Config, connection_string: &str) -> Result<Self> {
        let region = config.get_string("object_storage.region").unwrap();
        let endpoint = config.get_string("object_storage.endpoint").unwrap();
        let access_key = config.get_string("object_storage.access_key").unwrap();
        let secret_key = config.get_string("object_storage.secret_key").unwrap();

        let bucket = Bucket::new(
            "sandbox",
            Region::Custom {
                region,
                endpoint,
            },
            Credentials::new(Some(&access_key), Some(&secret_key), None, None, None).unwrap(),
        ).unwrap().with_path_style();

        Ok(Self {
            pool: PgPoolOptions::new()
                .connect(&connection_string)
                .await?,
            bucket,
        })
    }

    pub async fn new_task(&self, user_id: Option<String>, id: &TaskId, params: &TaskParams) {
        sqlx::query!(
            "insert into sandbox_tasks (user_id, task_id, is_pending, status, params) values ($1, $2, true, $3, $4)", 
            user_id, 
            id.as_str(),
            serde_json::to_value(PersistedTaskStatus::Pending).unwrap(),
            serde_json::to_value(match params {
                TaskParams::ImageGenerationParams { iterations, number_of_images, prompt } => PersistedTaskParamsV2::ImageGeneration {
                    iterations: *iterations,
                    number_of_images: *number_of_images,
                    prompt: prompt.clone(),
                },
            }).unwrap(),
        )
            .execute(&self.pool)
            .await
            .unwrap();
    }

    pub async fn get_user_tasks(&self, user_id: &str) -> Vec<Task> {
        let tasks = sqlx::query_as!(PersistedTask, "select task_id as id, status, created_at, params from sandbox_tasks where user_id = $1 order by created_at desc", user_id)
            .fetch_all(&self.pool)
            .await
            .unwrap();

        let mut result = Vec::new();

        for task in tasks {
            result.push(self.task_from_persisted_task(task).await);
        }

        result
    }

    pub async fn get_task(&self, id: &TaskId) -> Task {
        let task = sqlx::query_as!(PersistedTask, "select task_id as id, status, created_at, params from sandbox_tasks where task_id = $1", id.as_str())
            .fetch_one(&self.pool)
            .await
            .unwrap();

        self.task_from_persisted_task(task).await        
    }

    pub async fn get_any_new_task(&self) -> Option<Task> {
        let task = sqlx::query_as!(PersistedTask, "select task_id as id, status, created_at, params from sandbox_tasks where is_pending = true limit 1")
            .fetch_optional(&self.pool)
            .await
            .unwrap()?;

        Some(self.task_from_persisted_task(task).await)
    }

    async fn task_from_persisted_task(&self, task: PersistedTask) -> Task {
        let id = TaskId::new(task.id);
        let status = match serde_json::from_value::<PersistedTaskStatus>(task.status).unwrap() {
            PersistedTaskStatus::Pending => TaskStatus::Pending,
            PersistedTaskStatus::InProgress { current_step, total_steps, current_image } => TaskStatus::InProgress { 
                current_step, 
                total_steps, 
                current_image: current_image.unwrap_or(0),
            },
            PersistedTaskStatus::Finished => TaskStatus::Finished,
        };

        let created_at = NaiveDateTime::from_timestamp_opt(task.created_at.unix_timestamp(), 0).unwrap();
        let created_at = DateTime::from_utc(created_at, Utc);

        let params = match serde_json::from_value::<PersistedTaskParamsV2>(task.params.unwrap()).unwrap() {
            PersistedTaskParamsV2::ImageGeneration { 
                iterations, 
                number_of_images, 
                prompt
            } => TaskParams::ImageGenerationParams {
                prompt, 
                iterations, 
                number_of_images
            },
        };

        Task {
            id,
            status,
            created_at,
            params,
        }
    }

    pub async fn save_task_status(&self, id: &TaskId, status: &TaskStatus) {
        let persisted_status = match status {
            TaskStatus::Pending => PersistedTaskStatus::Pending,
            TaskStatus::InProgress { current_step, total_steps, current_image } => PersistedTaskStatus::InProgress { 
                current_step: *current_step, 
                total_steps: *total_steps,
                current_image: Some(*current_image),
            },
            TaskStatus::Finished => PersistedTaskStatus::Finished,
        };

        let is_pending = TaskStatus::Pending == *status;

        sqlx::query!(
            "update sandbox_tasks set status = $1::jsonb, is_pending = $2 where task_id = $3", 
            serde_json::to_value(&persisted_status).unwrap(),
            is_pending,
            id.as_str()
        )
            .execute(&self.pool)
            .await
            .unwrap();
    }

    pub async fn get_generated_image(&self, task_id: &TaskId) -> Option<Vec<u8>> {
        match self.bucket.get_object(&format!("output/images/{}", task_id.as_str())).await {
            Ok(v) => Some(v.to_vec()),
            Err(err) => match err {
                S3Error::HttpFailWithBody(code, _body) => match code {
                    404 => None,
                    other => panic!("failed to get generated image because of http error: {:?}", other),
                },
                other => panic!("failed to get generated image because of error: {:?}", other),
            },
        }
    }

    pub async fn create_or_get_user_by_email(&self, email: &str) -> UserId {
        let new_id = Ulid::new();

        let user_id = sqlx::query_as!(PersistedUserId, r#"
            with ins as (
                insert into sandbox_users (id, email) values ($1, $2) on conflict do nothing returning id
            )
            select id as "id!" from ins
            union all select id as "id!" from sandbox_users where email = $2 limit 1;
        "#, new_id.to_string(), email).fetch_one(&self.pool).await.unwrap();

        UserId::from_string(user_id.id)
    }

    pub async fn create_task_asset(&self, task_id: &TaskId, data: Vec<u8>) -> AssetId {
        let asset_id = Ulid::new();

        sqlx::query!("insert into sandbox_task_assets (task_id, asset_id) values ($1, $2)", task_id.as_str(), asset_id.to_string()).execute(&self.pool).await.unwrap();
        self.bucket.put_object(&format!("output/images/{}", asset_id.to_string()), &data).await.unwrap();

        AssetId::from_string(asset_id.to_string())
    }

    pub async fn get_task_assets(&self, task_id: &TaskId) -> Vec<AssetId> {
        sqlx::query_as!(PersistedAssetId, "select asset_id as id from sandbox_task_assets where task_id = $1", task_id.as_str())
            .fetch_all(&self.pool)
            .await
            .unwrap()
            .into_iter()
            .map(|v| AssetId::from_string(v.id))
            .collect()
    }

    pub async fn get_chat_messages(&self, task_id: &TaskId) -> Vec<ChatMessage> {
        let messages = sqlx::query_as!(PersistedChatMessage, "select task_id, message_id, content, message_role as \"message_role: _\", message_index from sandbox_chat_messages where task_id = $1 order by message_index desc", task_id.as_str())
            .fetch_all(&self.pool)
            .await
            .unwrap();

        messages
            .into_iter()
            .map(|v| ChatMessage {
                task_id: TaskId::new(v.task_id),
                message_id: MessageId::new(v.message_id),
                content: v.content,
                role: match v.message_role {
                    PersistedChatMessageRole::System => ChatMessageRole::System,
                    PersistedChatMessageRole::User => ChatMessageRole::User,
                    PersistedChatMessageRole::Assistant => ChatMessageRole::Assistant,
                },
                index: v.message_index as u32,
            })
            .collect()
    }

    pub async fn create_chat_message(&self, task_id: &TaskId, content: String, role: ChatMessageRole, index: u32) -> MessageId {
        let message_id = Ulid::new();

        sqlx::query!(
            "insert into sandbox_chat_messages (task_id, message_id, content, message_role, message_index) values ($1, $2, $3, $4, $5)",
            task_id.as_str(),
            message_id.to_string(),
            content,
            match role {
                ChatMessageRole::System => PersistedChatMessageRole::System,
                ChatMessageRole::User => PersistedChatMessageRole::User,
                ChatMessageRole::Assistant => PersistedChatMessageRole::Assistant,
            } as PersistedChatMessageRole,
            index as i32
        ).execute(&self.pool).await.unwrap();

        MessageId::new(message_id.to_string())
    }

    pub async fn append_chat_message(&self, task_id: &TaskId, content: String, role: ChatMessageRole) -> MessageId {
        let message_id = Ulid::new();

        sqlx::query!(
            "insert into sandbox_chat_messages (task_id, message_id, content, message_role, message_index) values ($1, $2, $3, $4, (select coalesce(max(message_index) + 1, 0) from sandbox_chat_messages where task_id = $1 and message_id = $2))",
            task_id.as_str(),
            message_id.to_string(),
            content,
            match role {
                ChatMessageRole::System => PersistedChatMessageRole::System,
                ChatMessageRole::User => PersistedChatMessageRole::User,
                ChatMessageRole::Assistant => PersistedChatMessageRole::Assistant,
            } as PersistedChatMessageRole
        ).execute(&self.pool).await.unwrap();

        MessageId::new(message_id.to_string())
    }

    pub async fn total_pending_tasks(&self) -> u64 {
        sqlx::query!("select count(*) as cnt from sandbox_tasks where is_pending = true")
            .fetch_one(&self.pool)
            .await
            .unwrap()
            .cnt
            .unwrap() as u64
    }

    pub async fn total_in_progress_tasks(&self) -> u64 {
        sqlx::query!("select count(*) as cnt from sandbox_tasks where status->'InProgress' is not null")
            .fetch_one(&self.pool)
            .await
            .unwrap()
            .cnt
            .unwrap() as u64
    }

    pub async fn finished_tasks_within_last_day(&self) -> u64 {
        sqlx::query!("select count(*) as cnt from sandbox_tasks where status::text = '\"Finished\"' and created_at > now() - interval '24' hour")
            .fetch_one(&self.pool)
            .await
            .unwrap()
            .cnt
            .unwrap() as u64
    }

    pub async fn get_max_task_pending_time(&self) -> Option<Duration> {
        sqlx::query!("select extract(epoch from max(now() - created_at))::int as lag from sandbox_tasks where is_pending = true")
            .fetch_one(&self.pool)
            .await
            .unwrap()
            .lag
            .map(|v| Duration::from_secs(v.try_into().unwrap()))
    }

    pub async fn update_worker_last_ping_time(&self) {
        sqlx::query!("update sandbox_workers set last_ping_at = now()")
            .execute(&self.pool)
            .await
            .unwrap();
    }

    pub async fn total_active_workers(&self) -> u64 {
        sqlx::query!("select count(*) as cnt from sandbox_workers where now() - last_ping_at < interval '10' minute")
            .fetch_one(&self.pool)
            .await
            .unwrap()
            .cnt
            .unwrap()
            .try_into()
            .unwrap()
    }
}

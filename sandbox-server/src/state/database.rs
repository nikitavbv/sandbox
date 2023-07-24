use {
    anyhow::Result,
    sqlx::{postgres::PgPoolOptions,types::time::OffsetDateTime},
    config::Config,
    serde::{Serialize, Deserialize},
    s3::{Bucket, creds::Credentials, region::Region, error::S3Error},
    ulid::Ulid,
    chrono::{NaiveDateTime, DateTime, Utc},
    crate::entities::{TaskId, TaskStatus, Task, UserId, AssetId},
};

struct PersistedTask {
    id: String,
    prompt: String,
    status: sqlx::types::JsonValue,
    created_at: OffsetDateTime,
}

#[derive(Serialize, Deserialize)]
enum PersistedTaskStatus {
    Pending,
    InProgress {
        current_step: u32,
        total_steps: u32,
    },
    Finished,
}

struct PersistedUserId {
    id: String,
}

struct PersistedAssetId {
    id: String,
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

    pub async fn new_task(&self, user_id: Option<String>, id: &TaskId, prompt: &str) {
        sqlx::query!(
            "insert into sandbox_tasks (user_id, task_id, prompt, is_pending, status) values ($1, $2, $3, true, $4)", 
            user_id, 
            id.as_str(),
            prompt,
            serde_json::to_value(PersistedTaskStatus::Pending).unwrap(),
        )
            .execute(&self.pool)
            .await
            .unwrap();
    }

    pub async fn get_user_tasks(&self, user_id: &str) -> Vec<Task> {
        let tasks = sqlx::query_as!(PersistedTask, "select task_id as id, prompt, status, created_at from sandbox_tasks where user_id = $1 order by created_at desc", user_id)
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
        let task = sqlx::query_as!(PersistedTask, "select task_id as id, prompt, status, created_at from sandbox_tasks where task_id = $1", id.as_str())
            .fetch_one(&self.pool)
            .await
            .unwrap();

        self.task_from_persisted_task(task).await        
    }

    pub async fn get_any_new_task(&self) -> Option<Task> {
        let task = sqlx::query_as!(PersistedTask, "select task_id as id, prompt, status, created_at from sandbox_tasks where is_pending = true limit 1")
            .fetch_optional(&self.pool)
            .await
            .unwrap()?;

        Some(self.task_from_persisted_task(task).await)
    }

    async fn task_from_persisted_task(&self, task: PersistedTask) -> Task {
        let id = TaskId::new(task.id);
        let status = match serde_json::from_value::<PersistedTaskStatus>(task.status).unwrap() {
            PersistedTaskStatus::Pending => TaskStatus::Pending,
            PersistedTaskStatus::InProgress { current_step, total_steps } => TaskStatus::InProgress { current_step, total_steps },
            PersistedTaskStatus::Finished => TaskStatus::Finished,
        };

        let created_at = NaiveDateTime::from_timestamp_opt(task.created_at.unix_timestamp(), 0).unwrap();
        let created_at = DateTime::from_utc(created_at, Utc);

        Task {
            id,
            prompt: task.prompt,
            status,
            created_at,
        }
    }

    pub async fn save_task_status(&self, id: &TaskId, status: &TaskStatus, image: Option<Vec<u8>>) {
        let persisted_status = match status {
            TaskStatus::Pending => PersistedTaskStatus::Pending,
            TaskStatus::InProgress { current_step, total_steps } => PersistedTaskStatus::InProgress { current_step: *current_step, total_steps: *total_steps },
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

        if let Some(image) = image {
            let asset_id = self.create_task_asset(id.as_str()).await;
            self.bucket.put_object(&format!("output/images/{}", asset_id.to_string()), &image).await.unwrap();
        }
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

    pub async fn create_task_asset(&self, task_id: &str) -> AssetId {
        let asset_id = Ulid::new();

        sqlx::query!("insert into sandbox_task_assets (task_id, asset_id) values ($1, $2)", task_id, asset_id.to_string()).execute(&self.pool).await.unwrap();

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
}

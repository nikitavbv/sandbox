use {
    anyhow::Result,
    sqlx::postgres::PgPoolOptions,
    config::Config,
    serde::{Serialize, Deserialize},
    s3::{Bucket, creds::Credentials, region::Region},
    sqlx::types::Uuid,
    ulid::Ulid,
    crate::entities::{TaskId, TaskStatus, Task, UserId},
};

struct PersistedTask {
    id: String,
    prompt: String,
    status: sqlx::types::JsonValue,
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
    id: Uuid,
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
        let tasks = sqlx::query_as!(PersistedTask, "select task_id as id, prompt, status from sandbox_tasks where user_id = $1", user_id)
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
        let task = sqlx::query_as!(PersistedTask, "select task_id as id, prompt, status from sandbox_tasks where task_id = $1", id.as_str())
            .fetch_one(&self.pool)
            .await
            .unwrap();

        self.task_from_persisted_task(task).await        
    }

    pub async fn get_any_new_task(&self) -> Option<Task> {
        let task = sqlx::query_as!(PersistedTask, "select task_id as id, prompt, status from sandbox_tasks where is_pending = true limit 1")
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

        Task {
            id,
            prompt: task.prompt,
            status,
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
            self.bucket.put_object(&format!("output/images/{}", id.as_str()), &image).await.unwrap();
        }
    }

    pub async fn get_generated_image(&self, task_id: &TaskId) -> Vec<u8> {
        let key = format!("output/images/{}", task_id.as_str());
        self.bucket.get_object(&key).await.unwrap().to_vec()
    }

    pub async fn create_or_get_user_by_email(&self, email: &str) -> UserId {
        let new_id = Ulid::new();

        let user_id = sqlx::query_as!(PersistedUserId, r#"
            with ins as (
                insert into sandbox_users (id, email) values ($1, $2) on conflict do nothing returning id
            )
            select id as "id!" from ins
            union all select id as "id!" from sandbox_users where email = $2 limit 1;
        "#, Uuid::parse_str(&new_id.to_string()).unwrap(), email).fetch_one(&self.pool).await.unwrap();

        UserId::from_u128(user_id.id.as_u128())
    }
}

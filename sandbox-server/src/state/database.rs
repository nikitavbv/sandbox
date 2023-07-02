use {
    anyhow::Result,
    sqlx::postgres::PgPoolOptions,
    config::Config,
    serde::{Serialize, Deserialize},
    s3::{Bucket, creds::Credentials, region::Region},
    crate::entities::{TaskId, TaskStatus, Task},
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
                .max_connections(2)
                .connect(&connection_string)
                .await?,
            bucket,
        })
    }

    /*pub async fn new_task(&self, user_id: Option<String>, id: &str, prompt: &str) -> Result<()> {
        sqlx::query!("insert into sandbox_tasks (user_id, task_id, prompt, status) values ($1, $2, $3, 'new')", user_id, id, prompt)
            .execute(&self.pool)
            .await
            .unwrap();
        Ok(())
    }*/

    /*pub async fn get_user_tasks(&self, user_id: &str) -> Vec<Task> {
        sqlx::query_as!(Task, "select task_id as id, prompt, status from sandbox_tasks where user_id = $1", user_id)
            .fetch_all(&self.pool)
            .await
            .unwrap()
    }*/

    /*pub async fn get_task(&self, id: &str) -> Task {
        let result = sqlx::query_as!(PromptAndStatus, "select prompt, status from sandbox_tasks where task_id = $1", id)
            .fetch_one(&self.pool)
            .await
            .unwrap();

        Task {
            id: id.to_owned(),
            prompt: result.prompt,
            status: result.status,
        }
    }*/

    pub async fn get_any_new_task(&self) -> Option<Task> {
        let task = sqlx::query_as!(PersistedTask, "select task_id as id, prompt, status from sandbox_tasks where is_pending = true limit 1")
            .fetch_optional(&self.pool)
            .await
            .unwrap()?;

        let id = TaskId::new(task.id);
        let status = match serde_json::from_value::<PersistedTaskStatus>(task.status).unwrap() {
            PersistedTaskStatus::Pending => TaskStatus::Pending,
            PersistedTaskStatus::InProgress { current_step, total_steps } => TaskStatus::InProgress { current_step, total_steps },
            PersistedTaskStatus::Finished => TaskStatus::Finished { image: self.get_generated_image(&id).await },
        };

        Some(Task {
            id,
            prompt: task.prompt,
            status,
        })
    }

    /*pub async fn get_any_new_task(&self) -> Option<Task> {
        let task = sqlx::query_as!(PersistedTask, "select task_id as id, prompt from sandbox_tasks where is_pending = true limit 1")
            .fetch_optional(&self.pool)
            .await
            .unwrap()?;

        Some(Task {
            id: TaskId::new(task.id),
            prompt: task.prompt,
            status: match task.status {
                PersistedTaskStatus::Pending => 
            },
        })
    }*/

    pub async fn save_task_status(&self, id: &TaskId, status: &TaskStatus) {
        let persisted_status = match status {
            TaskStatus::Pending => PersistedTaskStatus::Pending,
            TaskStatus::InProgress { current_step, total_steps } => PersistedTaskStatus::InProgress { current_step: *current_step, total_steps: *total_steps },
            TaskStatus::Finished { image: _ } => PersistedTaskStatus::Finished,
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

        if let TaskStatus::Finished { image } = status {
            self.bucket.put_object(&format!("output/images/{}", id.as_str()), image).await.unwrap();
        }

        unimplemented!()
    }

    pub async fn get_generated_image(&self, task_id: &TaskId) -> Vec<u8> {
        let key = format!("output/images/{}", task_id.as_str());
        self.bucket.get_object(&key).await.unwrap().to_vec()
    }
}

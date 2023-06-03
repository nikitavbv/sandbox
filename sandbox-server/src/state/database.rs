use {
    anyhow::Result,
    sqlx::postgres::PgPoolOptions,
};

pub struct Task {
    pub id: String,
    pub prompt: String,
    pub status: String,
}

pub struct PromptAndStatus {
    prompt: String,
    status: String,
}

pub struct Database {
    pool: sqlx::postgres::PgPool,
}

impl Database {
    pub async fn new(connection_string: &str) -> Result<Self> {
        Ok(Self {
            pool: PgPoolOptions::new()
                .max_connections(2)
                .connect(&connection_string)
                .await?,
        })
    }

    pub async fn new_task(&self, user_id: Option<String>, id: &str, prompt: &str) -> Result<()> {
        sqlx::query!("insert into sandbox_tasks (user_id, task_id, prompt, status) values ($1, $2, $3, 'new')", user_id, id, prompt)
            .execute(&self.pool)
            .await
            .unwrap();
        Ok(())
    }

    pub async fn get_user_tasks(&self, user_id: &str) -> Vec<Task> {
        sqlx::query_as!(Task, "select task_id as id, prompt, status from sandbox_tasks where user_id = $1", user_id)
            .fetch_all(&self.pool)
            .await
            .unwrap()
    }

    pub async fn get_task(&self, id: &str) -> Task {
        let result = sqlx::query_as!(PromptAndStatus, "select prompt, status from sandbox_tasks where task_id = $1", id)
            .fetch_one(&self.pool)
            .await
            .unwrap();

        Task {
            id: id.to_owned(),
            prompt: result.prompt,
            status: result.status,
        }
    }

    pub async fn get_any_new_task(&self) -> Option<Task> {
        sqlx::query_as!(Task, "select task_id as id, prompt, status from sandbox_tasks where status = 'new' limit 1")
            .fetch_optional(&self.pool)
            .await
            .unwrap()
    }

    pub async fn mark_task_as_complete(&self, id: &str) -> Result<()> {
        sqlx::query!("update sandbox_tasks set status = 'complete' where task_id = $1", id)
            .execute(&self.pool)
            .await
            .unwrap();
        Ok(())
    }
}
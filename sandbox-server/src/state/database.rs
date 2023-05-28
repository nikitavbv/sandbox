use {
    scylla::{Session, SessionBuilder},
    anyhow::Result,
    sqlx::postgres::PgPoolOptions,
};

pub struct Task {
    pub id: String,
    pub prompt: String,
    pub status: String,
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
        /*self.session.query("select task_id, prompt, status from sandbox.sandbox_tasks where user_id = ? allow filtering", (user_id,))
            .await
            .unwrap()
            .rows()
            .unwrap()
            .into_iter()
            .map(|v| v.into_typed::<(String, String, String)>().unwrap())
            .map(|v| Task { id: v.0, prompt: v.1, status: v.2 })
            .collect()*/
        Vec::new()
    }

    pub async fn get_task(&self, id: &str) -> Task {
        /*let (prompt, status) = self.session.query("select prompt, status from sandbox.sandbox_tasks where task_id = ?", (id,))
            .await
            .unwrap()
            .first_row()
            .unwrap()
            .into_typed::<(String, String)>()
            .unwrap();
        
        Task {
            id: id.to_owned(),
            prompt,
            status,
        }*/
        unimplemented!()
    }
}
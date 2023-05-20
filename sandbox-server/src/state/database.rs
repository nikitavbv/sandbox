use {
    scylla::{Session, SessionBuilder},
    anyhow::Result,
};

pub struct Task {
    pub prompt: String,
    pub status: String,
}

pub struct Database {
    session: Session,
}

impl Database {
    pub async fn new(node: &str) -> Result<Self> {
        Ok(Self {
            session: SessionBuilder::new()
                .known_node(node)
                .build()
                .await?,
        })
    }

    pub async fn new_task(&self, user_id: Option<String>, id: &str, prompt: &str) -> Result<()> {
        self.session.query("insert into sandbox.sandbox_tasks (user_id, task_id, prompt, status) values (?, ?, ?, 'new')", (user_id, id, prompt))
            .await?;
        Ok(())
    }

    pub async fn get_user_tasks(&self, user_id: &str) -> Vec<Task> {
        self.session.query("select prompt, status from sandbox.sandbox_tasks where user_id = ?", (user_id,))
            .await
            .unwrap()
            .rows()
            .unwrap()
            .into_iter()
            .map(|v| v.into_typed::<(String, String)>().unwrap())
            .map(|v| Task { prompt: v.0, status: v.1 })
            .collect()
    }

    pub async fn get_task(&self, id: &str) -> Task {
        let (prompt, status) = self.session.query("select prompt, status from sandbox.sandbox_tasks where task_id = ?", (id,))
            .await
            .unwrap()
            .first_row()
            .unwrap()
            .into_typed::<(String, String)>()
            .unwrap();
        
        Task {
            prompt,
            status,
        }
    }
}
use {
    scylla::{Session, SessionBuilder},
    anyhow::Result,
};

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

    pub async fn new_task(&self, id: &str, prompt: &str) -> Result<()> {
        self.session.query("insert into sandbox.sandbox_tasks (task_id, prompt, status) values (?, ?, 'new')", (id, prompt))
            .await?;
        Ok(())
    }
}
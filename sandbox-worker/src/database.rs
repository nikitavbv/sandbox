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

    pub async fn mark_task_as_complete(&self, id: &str) -> Result<()> {
        self.session.query("update sandbox.sandbox_tasks set status = 'complete' where task_id = ?", (id,))
            .await?;
        Ok(())
    }
}
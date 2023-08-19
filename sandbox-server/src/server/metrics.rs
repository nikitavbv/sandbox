use {
    std::time::Duration,
    tokio::time::sleep,
    prometheus::{Registry, IntGauge},
    crate::state::database::Database,
};

pub async fn collect_metrics(registry: Registry, database: &Database) {
    let total_pending_tasks = IntGauge::new("pending_tasks", "total tasks in pending state").unwrap();
    registry.register(Box::new(total_pending_tasks.clone())).unwrap();

    loop {
        sleep(Duration::from_secs(10)).await;

        total_pending_tasks.set(database.total_pending_tasks().await as i64);
    }
}
use {
    std::time::Duration,
    tokio::time::sleep,
    config::Config,
    prometheus::{Registry, TextEncoder, register_int_gauge_vec_with_registry, register_int_gauge_with_registry},
    crate::state::database::Database,
};

pub struct MetricsPushConfig {
    endpoint: String,
    username: String,
    password: String,
}

impl MetricsPushConfig {
    pub fn from_config(config: &Config) -> Self {
        Self {
            endpoint: config.get("metrics_push.endpoint").unwrap(),
            username: config.get("metrics_push.username").unwrap(),
            password: config.get("metrics_push.password").unwrap(),
        }
    }
}

pub async fn collect_metrics(registry: Registry, database: &Database) {
    let total_tasks_by_state = register_int_gauge_vec_with_registry!("tasks_state", "total tasks in pending state", &["state"], registry).unwrap();
    let task_pending_time_max = register_int_gauge_with_registry!("task_pending_time_max", "max pending time of all tasks in pending state", registry).unwrap();
    let workers_total_active = register_int_gauge_with_registry!("workers_active_total", "number of active workers", registry).unwrap();
    
    loop {
        sleep(Duration::from_secs(10)).await;

        total_tasks_by_state.with_label_values(&["pending"]).set(database.total_pending_tasks().await as i64);
        total_tasks_by_state.with_label_values(&["in_progress"]).set(database.total_in_progress_tasks().await as i64);
        total_tasks_by_state.with_label_values(&["finished"]).set(database.finished_tasks_within_last_day().await as i64);

        task_pending_time_max.set(database.get_max_task_pending_time().await.map(|v| v.as_secs()).unwrap_or(0).try_into().unwrap());

        workers_total_active.set(database.total_active_workers().await.try_into().unwrap());
    }
}

pub async fn push_metrics(config: MetricsPushConfig, registry: Registry) {
    let encoder = TextEncoder::new();
    let client = reqwest::Client::new();

    loop {
        sleep(Duration::from_secs(10)).await;

        let metrics = encoder.encode_to_string(&registry.gather()).unwrap();

        client.post(&config.endpoint)
            .basic_auth(&config.username, Some(&config.password))
            .body(metrics)
            .send()
            .await
            .unwrap();
    }
}
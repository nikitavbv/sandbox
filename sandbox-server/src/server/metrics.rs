use {
    std::time::Duration,
    tokio::time::sleep,
    config::Config,
    prometheus::{Registry, TextEncoder, register_int_gauge_with_registry},
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
    let total_pending_tasks = register_int_gauge_with_registry!("pending_tasks", "total tasks in pending state", registry).unwrap();

    loop {
        sleep(Duration::from_secs(10)).await;

        total_pending_tasks.set(database.total_pending_tasks().await as i64);
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
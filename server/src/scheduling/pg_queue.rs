use {
    std::{sync::Arc, time::Duration},
    async_trait::async_trait,
    tracing::info,
    sqlx::{postgres::PgPoolOptions, Pool, Postgres},
    rand::distributions::{Alphanumeric, Distribution},
    prost::Message,
    crate::{
        models::io::ModelData,
        scheduling::scheduler::Scheduler,
        context::Context,
    },
};

enum TaskStatus {
    Pending,
    InProgress,
    Completed,
}

impl TaskStatus {
    pub fn encode(&self) -> String {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::InProgress => "in-progress",
            TaskStatus::Completed => "completed",
        }.to_owned()
    }
}

pub struct PgQueueSchedulerClient {
    pool: Pool<Postgres>,
}

impl PgQueueSchedulerClient {
    pub async fn new(postgres_connection_string: &str) -> Self {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(postgres_connection_string)
            .await
            .unwrap();

        Self {
            pool,
        }
    }
}

#[async_trait]
impl Scheduler for PgQueueSchedulerClient {
    async fn run(&self, _context: Arc<Context>, model_id: &str, input: &ModelData) -> ModelData {
        let task_id = generate_task_id();
        let input: Vec<u8> = rpc::ModelData::from(input.clone()).encode_to_vec();

        sqlx::query("insert into sandbox_tasks (task_id, status, model_input, model_id) values ($1, $2, $3, $4)")
            .bind(&task_id)
            .bind(TaskStatus::Pending.encode())
            .bind(input)
            .bind(model_id)
            .execute(&self.pool)
            .await
            .unwrap();

        loop {
            let row: (Option<Vec<u8>>, ) = sqlx::query_as("select model_output from sandbox_tasks where task_id = $1 limit 1")
                .bind(&task_id)
                .fetch_one(&self.pool)
                .await
                .unwrap();

            if let Some(output) = row.0 {
                info!("got result: {}", output.len());
                let output = rpc::ModelData::decode(&*output).unwrap();
                return ModelData::from(output);
            } else {
                info!("waiting for result for task id: {}", task_id);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}


fn generate_task_id() -> String {
    let mut rng = rand::thread_rng();
    Alphanumeric.sample_iter(&mut rng)
        .take(14)
        .map(char::from)
        .collect()
}
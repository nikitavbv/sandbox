use {
    std::sync::Arc,
    async_trait::async_trait,
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
    New,
    Completed,
}

impl TaskStatus {
    pub fn encode(&self) -> String {
        match self {
            TaskStatus::New => "new",
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

        sqlx::query("insert into sanbox_tasks (task_id, status, model_input, model_id) values ($1, $2, $3, $5)")
            .bind(task_id)
            .bind(TaskStatus::New.encode())
            .bind(input)
            .bind(model_id)
            .execute(&self.pool)
            .await
            .unwrap();

        ModelData::new()
    }
}


fn generate_task_id() -> String {
    let mut rng = rand::thread_rng();
    Alphanumeric.sample_iter(&mut rng)
        .take(14)
        .map(char::from)
        .collect()
}
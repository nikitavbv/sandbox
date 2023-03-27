use {
    std::sync::Arc,
    async_trait::async_trait,
    crate::{
        models::io::ModelData,
        scheduling::scheduler::Scheduler,
        context::Context,
    },
};

pub struct DoNothingScheduler {
}

impl DoNothingScheduler {
    pub fn new() -> Self {
        Self {
        }
    }
}

#[async_trait]
impl Scheduler for DoNothingScheduler {
    async fn run(&self, _context: Arc<Context>, _model_id: &str, _input: &ModelData) -> ModelData {
        ModelData::new()
    }
}
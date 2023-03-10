use {
    std::sync::Arc,
    async_trait::async_trait,
    crate::{
        models::io::ModelData,
        context::Context,
    }
};

#[async_trait]
pub trait Scheduler {
    async fn run(&self, context: Arc<Context>, model_id: &str, input: &ModelData) -> ModelData;
}
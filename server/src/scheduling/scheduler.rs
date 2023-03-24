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

#[async_trait]
impl Scheduler for Box<dyn Scheduler + Send + Sync> {
    async fn run(&self, context: Arc<Context>, model_id: &str, input: &ModelData) -> ModelData {
        (**self).run(context, model_id, input).await
    }
}
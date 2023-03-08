use {
    std::sync::Arc,
    tokio::sync::Mutex,
    tracing::info,
    crate::{
        models::{ModelDefinition, Model, io::ModelData},
        scheduling::registry::ModelRegistry,
        context::Context,
    },
};

pub struct SimpleScheduler {
    registry: ModelRegistry,

    loaded_model: Mutex<Option<(ModelDefinition, Box<dyn Model + Send>)>>,
}

impl SimpleScheduler {
    pub fn new(registry: ModelRegistry) -> Self {
        Self {
            registry,
            loaded_model: Mutex::new(None),
        }
    }

    pub async fn run(&self, context: Arc<Context>, model_id: &str, input: &ModelData) -> ModelData {
        self.load_model_if_needed(context, model_id).await;
        let model = self.loaded_model.lock().await;
        model.as_ref().unwrap().1.run(input)
    }

    async fn load_model_if_needed(&self, context: Arc<Context>, model_id: &str) {
        let mut loaded_model = self.loaded_model.lock().await;

        if loaded_model.is_none() || loaded_model.as_ref().unwrap().0.id() != model_id {
            info!(model_id, "loading model");

            let definition = self.registry.get(model_id).unwrap();
            let model = definition.create_instance(context).await;
            *loaded_model = Some((definition, model));
        } else {
            info!(model_id, "model is already loaded");
        }
    }
}

use {
    std::sync::Arc,
    tokio::sync::Mutex,
    tracing::info,
    crate::{
        models::{ModelDefinition, Model, io::ModelData},
        scheduling::registry::ModelRegistry,
    },
};

pub struct SimpleScheduler {
    registry: ModelRegistry,

    // loaded_model: Mutex<Option<(ModelDefinition, Arc<Box<dyn Model>>)>>,
}

impl SimpleScheduler {
    pub fn new(registry: ModelRegistry) -> Self {
        Self {
            registry,
            // loaded_model: Mutex::new(None),
        }
    }

    pub async fn run(&self, model_id: &str, input: &ModelData) -> ModelData {
        unimplemented!()
        /*self.load_model_if_needed(model_id).await;
        let model = self.loaded_model.lock().await;
        model.as_ref().unwrap().1.run(input)*/
    }

    async fn load_model_if_needed(&self, model_id: &str) {
        /*let mut loaded_model = self.loaded_model.lock().await;

        if loaded_model.is_none() || loaded_model.as_ref().unwrap().0.id() != model_id {
            info!(model_id, "loading model");

            let definition = self.registry.get(model_id).unwrap();
            let model = definition.create_instance().await;
            *loaded_model = Some((definition, model));
        } else {
            info!(model_id, "model is already loaded");
        }*/
        unimplemented!()
    }
}
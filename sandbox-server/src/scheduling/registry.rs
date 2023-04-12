use {
    std::collections::HashMap,
    crate::models::ModelDefinition,
};

pub struct ModelRegistry {
    models: HashMap<String, ModelDefinition>,
}

impl ModelRegistry {
    pub async fn new() -> Self {
        Self {
            models: HashMap::new(),
        }
    }

    pub fn with_definition(self, definition: ModelDefinition) -> Self {
        let mut models = self.models;
        models.insert(definition.id().to_owned(), definition);

        Self {
            models,
            ..self
        }
    }

    pub fn get(&self, model_id: &str) -> Option<ModelDefinition> {
        self.models.get(model_id).cloned()
    }
}
use {
    crate::models::{io::ModelData, Model},
};

pub struct VideoHashesCompute {
}

impl VideoHashesCompute {
    fn new() -> Self {
        Self {
        }
    }
}

impl Model for VideoHashesCompute {
    fn run(&self, input: &ModelData) -> ModelData {
        ModelData::new()
    }
}
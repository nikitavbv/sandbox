use std::collections::HashMap;

pub struct ModelInput {
    data: HashMap<String, DataEntry>,
}

pub struct ModelOutput {
    data: HashMap<String, DataEntry>,
}

pub enum DataEntry {
    Text(String),
}

impl ModelInput {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn with_parameter(self, key: String, parameter: DataEntry) -> Self {
        let mut data = self.data;
        data.insert(key, parameter);

        Self {
            data,
            ..self
        }
    }

    pub fn with_text(self, key: String, value: String) -> Self {
        self.with_parameter(key, DataEntry::Text(value))
    }

    pub fn get_parameter(&self, key: &str) -> &DataEntry {
        self.data.get(key).as_ref().unwrap()
    }

    pub fn get_text(&self, key: &str) -> &str {
        match self.get_parameter(key) {
            DataEntry::Text(text) => text,
            other => panic!("parameter \"{}\" is not of type text", key),
        }
    }
}
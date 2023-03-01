use {
    std::collections::HashMap,
    rpc::InferenceRequest,
};

pub struct ModelData {
    data: HashMap<String, DataEntry>,
}

pub enum DataEntry {
    Text(String),
    Image(Vec<u8>),
}

impl ModelData {
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

    pub fn with_image(self, key: String, value: Vec<u8>) -> Self {
        self.with_parameter(key, DataEntry::Image(value))
    }

    pub fn get_parameter(&self, key: &str) -> &DataEntry {
        self.data.get(key).as_ref().unwrap()
    }

    pub fn get_text(&self, key: &str) -> &str {
        match self.get_parameter(key) {
            DataEntry::Text(text) => text,
            _ => panic!("parameter \"{}\" is not of type text", key),
        }
    }

    pub fn get_image(&self, key: &str) -> &Vec<u8> {
        match self.get_parameter(key) {
            DataEntry::Image(image) => image,
            _ => panic!("parameter \"{}\" is not of type text", key),
        }
    }
}

impl From<InferenceRequest> for ModelData {
    fn from(value: InferenceRequest) -> Self {
        Self {
            data: value.entries.into_iter()
                .map(|v| (v.key, DataEntry::from(v.value.unwrap())))
                .collect(),
        }
    }
}

impl From<rpc::data_entry::Value> for DataEntry {
    fn from(value: rpc::data_entry::Value) -> Self {
        match value {
            rpc::data_entry::Value::Text(text) => Self::Text(text),
            rpc::data_entry::Value::Image(image) => Self::Image(image),
        }
    }
}

impl From<ModelData> for Vec<rpc::DataEntry> {
    fn from(value: ModelData) -> Self {
        value.data
            .into_iter()
            .map(|v| rpc::DataEntry {
                key: v.0,
                value: Some(v.1.into()),
            })
            .collect()
    }
}

impl From<DataEntry> for rpc::data_entry::Value {
    fn from(value: DataEntry) -> Self {
        match value {
            DataEntry::Text(text) => rpc::data_entry::Value::Text(text),
            DataEntry::Image(image) => rpc::data_entry::Value::Image(image),
        }
    }
}
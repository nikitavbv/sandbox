use std::collections::HashMap;

pub struct ModelInput {
    data: HashMap<String, DataEntry>,
}

pub struct ModelOutput {
    data: HashMap<String, DataEntry>,
}

enum DataEntry {
    Text(String),
}
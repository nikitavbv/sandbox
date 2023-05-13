use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct TaskMessage {
    pub id: String,
    pub prompt: String,
}
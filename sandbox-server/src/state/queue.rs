use {
    serde::{Serialize, Deserialize},
    rdkafka::{
        config::ClientConfig,
        producer::FutureProducer,
    },
};

#[derive(Serialize, Deserialize)]
pub struct TaskMessage {
    id: String,
    prompt: String,
}

pub struct Queue {
    producer: FutureProducer,
}

impl Queue {
    pub async fn new(node: &str) -> Self {
        Self {
            producer: ClientConfig::new()
                .set("bootstrap.servers", node)
                .set("message.timeout.ms", "5000")
                .create()
                .unwrap(),
        }
    }
}
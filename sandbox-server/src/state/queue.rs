use {
    serde::{Serialize, Deserialize},
    rdkafka::{
        config::ClientConfig,
        producer::{FutureProducer, FutureRecord},
        util::Timeout,
    },
};

#[derive(Serialize, Deserialize)]
pub struct TaskMessage {
    pub id: String,
    pub prompt: String,
}

pub struct Queue {
    producer: FutureProducer,
}

impl Queue {
    pub fn new(node: &str) -> Self {
        Self {
            producer: ClientConfig::new()
                .set("bootstrap.servers", node)
                .set("message.timeout.ms", "5000")
                .create()
                .unwrap(),
        }
    }

    pub async fn publish_task_message(&self, message: &TaskMessage) {
        let payload = serde_json::to_vec(message).unwrap();
        let record = FutureRecord::to("sandbox-tasks")
            .key(&message.id)
            .payload(&payload);
        self.producer.send(record, Timeout::Never).await.unwrap();
    }
}
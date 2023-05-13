use rdkafka::{
    config::ClientConfig,
    producer::FutureProducer,
};

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
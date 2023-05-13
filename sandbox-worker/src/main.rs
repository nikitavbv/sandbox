use {
    std::time::Duration,
    tracing::{info, warn},
    rdkafka::{
        Message,
        config::ClientConfig,
        client::ClientContext,
        consumer::{StreamConsumer, ConsumerContext, Consumer, CommitMode},
    },
    tokio::time::sleep,
    sandbox_common::{
        utils::{init_logging, load_config},
        messages::TaskMessage,
    },
};

pub mod data;
pub mod model;

struct StreamingContext;

impl ClientContext for StreamingContext {
}

impl ConsumerContext for StreamingContext {
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();
    let config = load_config();
    
    info!("sandbox worker started");

    let consumer: StreamConsumer<StreamingContext> = ClientConfig::new()
        .set("group.id", "sandbox-worker")
        .set("bootstrap.servers", &config.get_string("queue.node").unwrap())
        .set("enable.auto.commit", "true")
        .create_with_context(StreamingContext)
        .unwrap();

    consumer.subscribe(&["sandbox-tasks"]).unwrap();

    loop {
        match consumer.recv().await {
            Err(e) => {
                warn!("kafka error: {:?}", e);
                sleep(Duration::from_secs(1)).await;
            },
            Ok(v) => {
                let payload = match v.payload_view::<[u8]>() {
                    None => {
                        warn!("no payload");
                        if let Err(err) = consumer.commit_message(&v, CommitMode::Async) {
                            warn!("failed to commit offsets: {:?}", err);
                        }
                        sleep(Duration::from_secs(1)).await;
                        continue;
                    },
                    Some(Ok(v)) => v,
                    Some(Err(e)) => {
                        warn!("error while reading payload: {:?}", e);
                        if let Err(err) = consumer.commit_message(&v, CommitMode::Async) {
                            warn!("failed to commit offsets: {:?}", err);
                        }
                        sleep(Duration::from_secs(1)).await;
                        continue;
                    }
                };

                let payload: TaskMessage = match serde_json::from_slice(&payload) {
                    Ok(v) => v,
                    Err(err) => {
                        warn!("error while deserializing payload: {:?}", err);
                        if let Err(err) = consumer.commit_message(&v, CommitMode::Async) {
                            warn!("failed to commit offsets: {:?}", err);
                        }
                        sleep(Duration::from_secs(1)).await;
                        continue;
                    }
                };

                info!("generate image for prompt: {}", payload.prompt);

                if let Err(err) = consumer.commit_message(&v, CommitMode::Async) {
                    warn!("failed to commit offsets: {:?}", err);
                }
            }
        }
    }
}

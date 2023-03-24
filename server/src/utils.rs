use {
    tracing::Level,
    tracing_subscriber::{
        prelude::*,
        filter::filter_fn,
    },
};

pub fn init_logging() {
    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish()
        .with(filter_fn(|metadata| {
            if metadata.target().starts_with("sqlx::query") {
                metadata.level() > &Level::INFO
            } else {
                true
            }
        }))
        .init();
}

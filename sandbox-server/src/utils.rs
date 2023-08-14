use {
    std::env::var,
    tracing::Level,
    tracing_subscriber::{
        prelude::*,
        filter::filter_fn,
    },
    config::Config,
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

pub fn load_config() -> Config {
    Config::builder()
        .add_source(config::File::with_name(var("SANDBOX_CONFIG_PATH").unwrap_or("./config.toml".to_owned()).as_str()))
        .add_source(config::Environment::with_prefix("SANDBOX"))
        .build()
        .unwrap()
}
use {
    tracing::info,
    sandbox_common::utils::init_logging,
};

fn main() {
    init_logging();
    
    info!("sandbox worker started");
}

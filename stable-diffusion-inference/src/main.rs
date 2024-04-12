#[tokio::main]
async fn main() {
    init_logging();

    let addr = "0.0.0.0:8080";
    let model = StableDiffusionImageGenerationModel::load();
    let state = AppState {
        model: Arc::new(Mutex::new(model)),
    };

    info!("starting stable diffusion inference server on {}", addr);

    let app = Router::new()
        .route("/", get(root))
        .fallback(not_found_handler)
        .with_state(state);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await
}

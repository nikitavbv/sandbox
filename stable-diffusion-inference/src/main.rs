use {
    std::sync::Arc,
    tokio::{sync::Mutex, net::TcpListener},
    tracing::{info, warn, Level},
    tracing_subscriber::FmtSubscriber,
    axum::{
        Router,
        routing::get,
        extract::Request,
        body::Body,
        response::IntoResponse,
        http::StatusCode,
    },
    crate::model::ImageGenerationModel,
};

mod model;

#[derive(Clone)]
struct AppState {
    model: Arc<Mutex<ImageGenerationModel>>,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
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

async fn root() -> &'static str {
    "Hello, World!"
}

async fn not_found_handler(req: Request<Body>) -> impl IntoResponse {
    warn!(
        url = req.uri().to_string(),
        method = req.method().to_string(),
        "endpoint is not implemented"
    );
    (StatusCode::NOT_FOUND, "endpoint not implemented\n")
}

fn init_logging() {
    FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .event_format(tracing_subscriber::fmt::format::json())
        .init();
}

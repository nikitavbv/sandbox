use {
    serde::Deserialize,
    axum::{Router, extract::Path, routing::get},
};

#[derive(Deserialize)]
pub struct AssetID {
    pub asset_id: String,
}

pub fn rest_router() -> Router {
    Router::new()
        .route("/storage/:asset_id", get(serve_asset))
}

async fn serve_asset(Path(asset_id): Path<AssetID>) -> Vec<u8> {
    unimplemented!()
}
use {
    std::sync::Arc,
    serde::Deserialize,
    axum::{Router, Extension, response::Response, extract::Path, routing::get, http::header::{CONTENT_TYPE, HeaderValue}, body::Body},
    crate::{
        entities::TaskId,
        state::database::Database,
    },
};

#[derive(Deserialize, Debug)]
pub struct AssetID {
    pub asset_id: String,
}

pub fn rest_router(database: Arc<Database>) -> Router {
    Router::new()
        .route("/storage/:asset_id", get(serve_asset))
        .layer(Extension(database))
}

async fn serve_asset(Extension(database): Extension<Arc<Database>>, Path(asset_id): Path<AssetID>) -> Response<Body> {
    let body = database.get_generated_image(&TaskId::new(asset_id.asset_id)).await;

    let mut res = Response::new(Body::from(body));
    res.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("image/png"));
    res
}
use {
    std::sync::Arc,
    serde::Deserialize,
    axum::{
        Router, 
        Extension,
        response::Response,
        extract::Path,
        routing::get, 
        http::{StatusCode, header::{CONTENT_TYPE, HeaderValue}}, 
        body::Body,
    },
    crate::{
        entities::TaskId,
        state::database::Database,
    },
};

#[derive(Deserialize, Debug)]
pub struct AssetID {
    pub asset_id: String,
}

pub fn rest_router(database: Arc<Database>, encoding_key: jsonwebtoken::EncodingKey) -> Router {
    Router::new()
        .route("/v1/storage/:asset_id", get(serve_asset))
        .layer(Extension(database))
        .layer(Extension(encoding_key))
}

async fn serve_asset(Extension(database): Extension<Arc<Database>>, Path(asset_id): Path<AssetID>) -> Response<Body> {
    let body = match database.get_generated_image(&TaskId::new(asset_id.asset_id)).await {
        Some(v) => v,
        None => {
            let mut res = Response::new(Body::from("not_found"));
            *res.status_mut() = StatusCode::NOT_FOUND;
            return res;
        }
    };

    let mut res = Response::new(Body::from(body));
    res.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("image/png"));
    res
}

use {
    std::sync::Arc,
    serde::Deserialize,
    axum::{
        Router, 
        Extension,
        response::{Response, IntoResponse}, 
        extract::Path, 
        routing::get, 
        http::header::{CONTENT_TYPE, HeaderValue}, 
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

pub fn rest_router(database: Arc<Database>) -> Router {
    Router::new()
        .route("/v1/storage/:asset_id", get(serve_asset))
        .route("/v1/auth/callback", get(auth_callback))
        .layer(Extension(database))
}

async fn serve_asset(Extension(database): Extension<Arc<Database>>, Path(asset_id): Path<AssetID>) -> Response<Body> {
    let body = database.get_generated_image(&TaskId::new(asset_id.asset_id)).await;

    let mut res = Response::new(Body::from(body));
    res.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("image/png"));
    res
}

async fn auth_callback() -> impl IntoResponse {
    let client = reqwest::Client::new();

    // TODO: finish this
    /*let res = client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", redirect_uri()),
        ])
        .send()
        .await;*/

    unimplemented!()
}
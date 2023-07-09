use {
    std::sync::Arc,
    serde::{Serialize, Deserialize},
    tracing::error,
    axum::{
        Router, 
        Extension,
        response::{Response, IntoResponse}, 
        extract::{Path, Query}, 
        routing::get, 
        http::header::{CONTENT_TYPE, HeaderValue}, 
        body::Body,
    },
    jsonwebtoken::Algorithm,
    chrono::Utc,
    crate::{
        entities::TaskId,
        state::database::Database,
    },
};

#[derive(Deserialize, Debug)]
pub struct AssetID {
    pub asset_id: String,
}

#[derive(Clone)]
pub struct OAuthClientSecret {
    pub client_secret: String,
}

#[derive(Deserialize)]
pub struct OAuthCallbackData {
    code: String,
}

#[derive(Deserialize)]
struct OAuthCodeExchangeResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct UserInfoResponse {
    id: String,
    email: String,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenClaims {
    exp: usize,
    sub: String,
    email: String,
    name: String,
}

pub fn rest_router(database: Arc<Database>, oauth_client_secret: OAuthClientSecret, encoding_key: jsonwebtoken::EncodingKey) -> Router {
    Router::new()
        .route("/v1/storage/:asset_id", get(serve_asset))
        .route("/v1/auth/callback", get(auth_callback))
        .layer(Extension(database))
        .layer(Extension(oauth_client_secret))
        .layer(Extension(encoding_key))
}

async fn serve_asset(Extension(database): Extension<Arc<Database>>, Path(asset_id): Path<AssetID>) -> Response<Body> {
    let body = database.get_generated_image(&TaskId::new(asset_id.asset_id)).await;

    let mut res = Response::new(Body::from(body));
    res.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("image/png"));
    res
}

async fn auth_callback(
    Extension(client_secret): Extension<OAuthClientSecret>, 
    Extension(encoding_key): Extension<jsonwebtoken::EncodingKey>,
    data: Query<OAuthCallbackData>) -> impl IntoResponse {
    let client = reqwest::Client::new();

    // TODO: finish this
    let res = client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("client_id", "916750455653-biu6q4c7llj7q1k14h3qaquktcdlkeo4.apps.googleusercontent.com"),
            ("client_secret", &client_secret.client_secret),
            ("code", &data.code),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await;

    let res = match res {
        Ok(v) => v,
        Err(err) => {
            error!("failed to run code exchange request: {:?}", err);
            return "something went wrong".into_response();
        }
    };

    let res: OAuthCodeExchangeResponse = match res.json().await {
        Ok(v) => v,
        Err(err) => {
            error!("failed to get code exchange response: {:?}", err);
            return "failed to get token exchange response".into_response();
        }
    };

    let res = client
        .get("https://www.googleapis.com/oauth2/v1/userinfo")
        .bearer_auth(res.access_token)
        .send()
        .await;

    let res = match res {
        Ok(v) => v,
        Err(err) => {
            error!("failed to request user info: {:?}", err);
            return "failed to request user info".into_response();
        }
    };

    let res: UserInfoResponse = match res.json().await {
        Ok(v) => v,
        Err(err) => {
            error!("failed to get user info response: {:?}", err);
            return "failed to get user info response".into_response();
        }
    };

    let token = issue_token(encoding_key, &res.id, &res.email, &res.name);

    // TODO: redirect with issued token
    // let mut redirect_to = Url::parse("https://sandbox.nikitavbv.com/");

    unimplemented!()
}

fn issue_token(encoding_key: jsonwebtoken::EncodingKey, id: &str, email: &str, name: &str) -> String {
    jsonwebtoken::encode(
        &jsonwebtoken::Header::new(Algorithm::RS384),
        &TokenClaims {
            exp: (Utc::now().timestamp() as usize) + (7 * 24 * 60 * 60),
            sub: format!("google:{}", id),
            email: email.to_owned(),
            name: name.to_owned(),
        },
        &encoding_key
    ).unwrap()
}
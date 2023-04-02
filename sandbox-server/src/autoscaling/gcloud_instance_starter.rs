use {
    tracing::info,
    config::Config,
    gcp_auth::{AuthenticationManager, CustomServiceAccount},
    hyper_tls::HttpsConnector,
    async_trait::async_trait,
    crate::scheduling::scheduler::Scheduler,
};

pub struct GcloudInstanceStarter {
    service_account: CustomServiceAccount,
}

impl GcloudInstanceStarter {
    pub fn new(service_account: CustomServiceAccount) -> Self {
        Self {
            service_account,
        }
    }
}

/*#[async_trait]
impl Scheduler for GcloudInstanceStarter {
    async fn run(&self, context: Arc<Context>, model_id: &str, input: &ModelData) -> ModelData {
        // TODO: implement scheduler that will start google cloud instance if it is not running yet.
    }
}*/

pub async fn start(config: &Config) {
    info!("this is autoscaling test");

    use base64::Engine;

    let key = config.get_string("autoscaling.gcp_key").unwrap();
    let key = base64::engine::general_purpose::STANDARD_NO_PAD.decode(key).unwrap();
    let key = String::from_utf8_lossy(&key);
    let service_account = CustomServiceAccount::from_json(&key).unwrap();

    let auth_manager = gcp_auth::AuthenticationManager::from(service_account);
    let scopes = &["https://www.googleapis.com/auth/compute"];
    let token = auth_manager.get_token(scopes).await.unwrap();
    let token = token.as_str();

    let https = HttpsConnector::new();
    let client = hyper::Client::builder().build::<_, hyper::Body>(https);
    let request = hyper::Request::builder()
        .uri("https://compute.googleapis.com/compute/v1/projects/nikitavbv/zones/europe-central2-b/instances/8951149891710854966/start")
        .method(hyper::http::Method::POST)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Length", "0")
        .body(hyper::Body::empty())
        .unwrap();
    let res = client.request(request).await.unwrap();

    info!("status is: {}", res.status());
}
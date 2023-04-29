use {
    std::sync::Arc,
    gcp_auth::{AuthenticationManager, CustomServiceAccount},
    hyper_tls::HttpsConnector,
    async_trait::async_trait,
    crate::{
        context::Context,
    },
};

pub struct GcloudInstanceStarter {
    auth_manager: AuthenticationManager,
}

impl GcloudInstanceStarter {
    pub fn new(service_account: CustomServiceAccount) -> Self {
        Self {
            auth_manager: gcp_auth::AuthenticationManager::from(service_account),
        }
    }

    fn start(&self) {
        /*let scopes = &["https://www.googleapis.com/auth/compute"];
        let token = self.auth_manager.get_token(scopes).await.unwrap();
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
        client.request(request).await.unwrap();

        self.inner.run(context, model_id, input).await*/
    }
}
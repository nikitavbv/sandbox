use {
    web_sys::window,
    tonic_web_wasm_client::Client,
    yew_router::prelude::*,
    gloo_storage::{LocalStorage, Storage},
    tonic::{
        metadata::MetadataValue, 
        Status, 
        Request, 
        service::Interceptor,
        codegen::InterceptedService,
    },
    rpc::sandbox_service_client::SandboxServiceClient,
};

pub type SandboxClient = SandboxServiceClient<InterceptedService<Client, AuthTokenSetterInterceptor>>;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/login")]
    Login,
    #[at("/tasks/:id")]
    Task { id: String },
    #[at("/history")]
    History,
}

pub fn client() -> SandboxClient {
    client_with_token(LocalStorage::get("access_token").ok())
}

pub fn client_with_token(token: Option<String>) -> SandboxClient {
    SandboxServiceClient::with_interceptor(
        Client::new(format!("{}/api", window().unwrap().location().origin().unwrap())),
        AuthTokenSetterInterceptor::new(token),
    )
}

pub struct AuthTokenSetterInterceptor {
    token: Option<String>,
}

impl AuthTokenSetterInterceptor {
    pub fn new(token: Option<String>) -> Self {
        Self {
            token,
        }
    }
}

impl Interceptor for AuthTokenSetterInterceptor {
    fn call(&mut self, mut req: Request<()>) -> Result<Request<()>, Status> {
        if let Some(token) = self.token.as_ref() {
            req.metadata_mut().insert("x-access-token", MetadataValue::try_from(token).unwrap());
        }
        Ok(req)
    }
}
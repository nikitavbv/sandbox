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
    stylist::Style,
    yew::Classes,
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
        Client::new(format!("{}/v1/rpc", window().unwrap().location().origin().unwrap())),
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

pub struct MultiClass {
    classes: Vec<String>,
}

impl MultiClass {
    pub fn new() -> Self {
        Self {
            classes: Vec::new(),
        }
    }

    pub fn with(self, class: &Style) -> Self {
        let mut classes = self.classes;
        classes.push(class.get_class_name().to_owned());

        Self {
            classes,
        }
    }
}

impl From<MultiClass> for Classes {
    fn from(value: MultiClass) -> Self {
        value.classes.join(" ").into()
    }
}
use {
    web_sys::window,
    tonic_web_wasm_client::Client,
    yew_router::prelude::*,
    rpc::{
        ml_sandbox_service_client::MlSandboxServiceClient,
        GenerateImageRequest,
    },
};

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/login")]
    Login,
    #[at("/tasks/:id")]
    Task { id: String },
}

pub fn client() -> MlSandboxServiceClient<Client> {
    MlSandboxServiceClient::new(Client::new(format!("{}/api", window().unwrap().location().origin().unwrap())))
}
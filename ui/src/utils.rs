use {
    web_sys::window,
    tonic_web_wasm_client::Client,
    rpc::{
        ml_sandbox_service_client::MlSandboxServiceClient,
        GenerateImageRequest,
    },
};

pub fn client() -> MlSandboxServiceClient<Client> {
    MlSandboxServiceClient::new(Client::new(format!("{}/api", window().unwrap().location().origin().unwrap())))
}
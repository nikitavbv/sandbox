use {
    tracing::info,
    yew::prelude::*,
    tonic_web_wasm_client::Client,
    tonic::{Request, Status},
    wasm_bindgen_futures::spawn_local,
    tracing_wasm::WASMLayerConfigBuilder,
    rpc::{
        ml_sandbox_service_client::MlSandboxServiceClient,
        RunSimpleModelRequest,
    }
};

#[function_component(App)]
fn app() -> Html {
    info!("application started");

    let on_file_change = {

        Callback::from(move |e: InputEvent| {
            info!("callback is called");

            // TODO: implement reading file here
        })
    };

    html!(
        <div>
            {"Hello ML Sandbox!"}
            <input type="file" oninput={on_file_change} />
        </div>
    )
}

fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default_with_config(
        WASMLayerConfigBuilder::new()
            .set_max_level(tracing::Level::INFO)
            .build()
        );
    yew::Renderer::<App>::new().render();
}
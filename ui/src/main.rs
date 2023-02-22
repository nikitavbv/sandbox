use {
    tracing::info,
    yew::prelude::*,
    yew_router::prelude::*,
    tonic_web_wasm_client::Client,
    tonic::{Request, Status},
    wasm_bindgen_futures::spawn_local,
    tracing_wasm::WASMLayerConfigBuilder,
    rpc::{
        ml_sandbox_service_client::MlSandboxServiceClient,
        RunSimpleModelRequest,
    }
};

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/models/image-classification")]
    ImageClassificationModel,
    #[at("/models/image-generation")]
    ImageGenerationModel,
    #[at("/models/text-generation")]
    TextGenerationModel,
}

#[function_component(App)]
fn app() -> Html {
    info!("application started");

    html!(
        <div>
            {"Hello ML Sandbox!"}
            <BrowserRouter>
                <Switch<Route> render={router_switch} />
            </BrowserRouter>
        </div>
    )
}

fn router_switch(route: Route) -> Html {
    match route {
        Route::Home => html!(<Home />),
        Route::ImageClassificationModel => html!(<ImageClassificationModel />),
        Route::ImageGenerationModel => html!(<ImageGenerationModel />),
        Route::TextGenerationModel => html!(<TextGenerationModel />),
    }
}

#[function_component(Home)]
fn home() -> Html {
    let navigator = use_navigator().unwrap();

    html!(
        <div>
            {"Home"}
            <button>{"image generation model"}</button>
            <button>{"text generation model"}</button>
        </div>
    )
}

#[function_component(ImageClassificationModel)]
fn image_classification_model() -> Html {
    let on_file_change = {

        Callback::from(move |e: InputEvent| {
            info!("callback is called");

            // TODO: implement reading file here
        })
    };

    html!(
        <>
            <input type="file" oninput={on_file_change} />
        </>
    )
}

#[function_component(ImageGenerationModel)]
fn run_image_generation_model() -> Html {
    html!(<div>{"image generation model"}</div>)
}

#[function_component(TextGenerationModel)]
fn text_generation_model() -> Html {
    html!(<div>{"text generation model"}</div>)
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
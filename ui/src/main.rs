use {
    std::{sync::{Arc, Mutex}, rc::Rc},
    tracing::info,
    yew::prelude::*,
    yew_router::prelude::*,
    tonic_web_wasm_client::Client,
    tonic::{Request, Status},
    wasm_bindgen_futures::spawn_local,
    tracing_wasm::WASMLayerConfigBuilder,
    web_sys::{EventTarget, HtmlInputElement},
    wasm_bindgen::JsCast,
    rpc::{
        ml_sandbox_service_client::MlSandboxServiceClient,
        RunSimpleModelRequest,
        InferenceRequest,
        DataEntry,
        data_entry
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

#[derive(Clone)]
struct ModelState {
    inference_started: bool,
    prompt: String,
    result: Option<InferenceResult>,
    result_generated_by: Option<String>,
}

#[derive(Clone, PartialEq)]
enum InferenceResultData {
    Text(String),
    Image(Vec<u8>),
}

#[derive(Clone, PartialEq)]
struct InferenceResult {
    data: InferenceResultData,
    worker: String,
}

enum ModelAction {
    UpdatePrompt(String),
    StartInference,
    SetInferenceResult(InferenceResult),
}

#[derive(Properties, PartialEq)]
pub struct InferenceResultDisplayProps {
    result: InferenceResult,
}

impl Default for ModelState {
    fn default() -> Self {
        Self {
            inference_started: false,
            prompt: "".to_owned(),
            result: None,
            result_generated_by: None,
        }
    }
}

impl Reducible for ModelState {
    type Action = ModelAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            Self::Action::UpdatePrompt(prompt) => Self {
                prompt,
                ..(*self).clone()
            },
            Self::Action::StartInference => Self {
                inference_started: true,
                result: None,
                ..(*self).clone()
            },
            Self::Action::SetInferenceResult(result) => Self {
                inference_started: false,
                result: Some(result),
                ..(*self).clone()
            },
        }.into()
    }
}

#[function_component(App)]
fn app() -> Html {
    info!("application started");

    html!(
        <div>
            {"ML Sandbox"}
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
    let navigator = Arc::new(use_navigator().unwrap());

    let image_generation_btn_handler = {
        let navigator = navigator.clone();

        Callback::from(move |_| navigator.push(&Route::ImageGenerationModel))
    };

    let text_generation_btn_handler = Callback::from(move |_| navigator.push(&Route::TextGenerationModel));

    html!(
        <div>
            {"Home"}
            <button onclick={image_generation_btn_handler}>{"image generation model"}</button>
            <button onclick={text_generation_btn_handler}>{"text generation model"}</button>
        </div>
    )
}

#[function_component(ImageClassificationModel)]
fn image_classification_model() -> Html {
    let on_file_change = {

        Callback::from(move |_e: InputEvent| {
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
    let navigator = use_navigator().unwrap();
    let client = Arc::new(Mutex::new(client()));
    let state = use_reducer(ModelState::default);

    let go_home_btn_handler = Callback::from(move |_| navigator.push(&Route::Home));
    let on_prompt_change = {
        let state = state.clone();

        Callback::from(move |e: Event| {
            let target: Option<EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
            if let Some(input) = input {
                state.dispatch(ModelAction::UpdatePrompt(input.value()));
            }
        })
    };

    let run_inference = {
        let state = state.clone();
        let client = client.clone();

        let prompt = state.prompt.clone();

        Callback::from(move |_| {
            let client = client.clone();
            let state = state.clone();
            let prompt = prompt.clone();

            {
                let state = state.clone();

                spawn_local(async move {
                    let mut client = client.lock().unwrap();
                    let res = client.run_image_generation_model(InferenceRequest {
                        entries: vec![DataEntry {
                            key: "prompt".to_owned(),
                            value: Some(data_entry::Value::Text(prompt.clone())),
                        }]
                    }).await.unwrap().into_inner();

                    state.dispatch(ModelAction::SetInferenceResult(InferenceResult {
                        data: InferenceResultData::Image(res.image),
                        worker: res.worker,
                    }));
                });
            }

            state.dispatch(ModelAction::StartInference);
        })
    };

    let result = if let Some(result) = &state.result {
        html!(<InferenceResultDisplay result={result.clone()} />)
    } else {
        html!(<div></div>)
    };

    html!(
        <div>
            <button onclick={go_home_btn_handler}>{"home"}</button>
            <h1>{"image generation model"}</h1>
            <input onchange={on_prompt_change} value={state.prompt.clone()} placeholder={"prompt"}/>
            <button onclick={run_inference}>{"run model"}</button>
            { result }
        </div>
    )
}
 
#[function_component(TextGenerationModel)]
fn text_generation_model() -> Html {
    let navigator = use_navigator().unwrap();
    let client = Arc::new(Mutex::new(client()));
    let state = use_reducer(ModelState::default);

    let go_home_btn_handler = Callback::from(move |_| navigator.push(&Route::Home));
    let on_prompt_change = {
        let state = state.clone();

        Callback::from(move |e: Event| {
            let target: Option<EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
            if let Some(input) = input {
                state.dispatch(ModelAction::UpdatePrompt(input.value()));
            }
        })
    };

    let run_inference = {
        let state = state.clone();
        let client = client.clone();
        
        let prompt = state.prompt.clone();

        Callback::from(move |_| {
            let client = client.clone();
            let prompt = prompt.clone();

            {
                let state = state.clone();

                spawn_local(async move {
                    let mut client = client.lock().unwrap();
                    let res = client.run_text_generation_model(InferenceRequest {
                        entries: vec![DataEntry {
                            key: "prompt".to_owned(),
                            value: Some(data_entry::Value::Text(prompt.clone())),
                        }],
                    }).await.unwrap().into_inner();
    
                    state.dispatch(ModelAction::SetInferenceResult(InferenceResult {
                        data: InferenceResultData::Text(res.text),
                        worker: res.worker,
                    }));
                });
            }

            state.dispatch(ModelAction::StartInference);
        })
    };

    let model_controls = if state.inference_started {
        html!({"running inference..."})
    } else {
        html!(<button onclick={run_inference}>{"run model"}</button>)
    };

    let result = if let Some(result) = &state.result {
        html!(<InferenceResultDisplay result={result.clone()} />)
    } else {
        html!(<div></div>)
    };

    html!(
        <div>
            <button onclick={go_home_btn_handler}>{"home"}</button>
            <h1>{"text generation model"}</h1>
            <input onchange={on_prompt_change} value={state.prompt.clone()} placeholder={"prompt"}/>
            { model_controls }
            { result }
        </div>
    )
}

#[function_component(InferenceResultDisplay)]
fn inference_result_display(props: &InferenceResultDisplayProps) -> Html {
    match &props.result.data {
        InferenceResultData::Text(text) => html!(
            <div>
                <div><b>{"Result: "}</b>{ text }</div>
                <div><b>{"Generated by "}</b>{ &props.result.worker }</div>
            </div>
        ),
        InferenceResultData::Image(image) => html!(
            <div>
                <img src={format!("data:image/png;base64, {}", base64::encode(image))} style={"display: block;"} />
                <div><b>{"Generated by "}</b>{ &props.result.worker }</div>
            </div>
        ),
    }
}

fn client() -> MlSandboxServiceClient<Client> {
    MlSandboxServiceClient::new(Client::new("https://sandbox.nikitavbv.com".to_owned()))
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
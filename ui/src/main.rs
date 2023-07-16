use {
    std::{sync::{Arc, Mutex}, rc::Rc, collections::HashMap},
    tracing::info,
    yew::prelude::*,
    yew_router::{prelude::*, navigator},
    tonic::{Request, Status},
    wasm_bindgen_futures::spawn_local,
    tracing_wasm::WASMLayerConfigBuilder,
    web_sys::{EventTarget, HtmlInputElement, window},
    wasm_bindgen::JsCast,
    urlencoding::encode,
    gloo_storage::{Storage, LocalStorage},
    stylist::{style, yew::styled_component},
    rpc::{
        sandbox_service_client::SandboxServiceClient,
        CreateTaskRequest,
    },
    crate::{
        components::header::Header,
        pages::{
            task::TaskPage,
            login::LoginPage,
            history::HistoryPage,
        },
        utils::{client_with_token, Route},
    },
};

pub mod components;
pub mod pages;
pub mod utils;

#[derive(Clone)]
struct ModelState {
    inference_started: bool,
    prompt: String,
    result: Option<InferenceResult>,
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

#[styled_component(App)]
fn app() -> Html {
    html!(
        <div>
            <BrowserRouter>
                <Switch<Route> render={router_switch} />
            </BrowserRouter>
        </div>
    )
}

fn router_switch(route: Route) -> Html {
    html!(<RouterComponent route={route} />)
}

#[derive(Properties, PartialEq)]
struct RouterComponentProps {
    route: Route,
}

#[function_component(RouterComponent)]
fn router_component(props: &RouterComponentProps) -> Html {
    let style = style!(r#"
        padding: 24px;
    "#).unwrap();

    let is_logged_in = use_state(|| LocalStorage::get::<String>("access_token").is_ok());
    let logout = {
        let is_logged_in_setter = is_logged_in.setter();
        
        move |_| {
            LocalStorage::delete("access_token");
            is_logged_in_setter.set(false);
        }
    };
    let login = {
        let is_logged_in_setter = is_logged_in.setter();

        move |_: ()| {
            is_logged_in_setter.set(true);
        }
    };

    let body = match &props.route {
        Route::Home => html!(<Home />),
        Route::Login => html!(<LoginPage login={login} />),
        Route::Task { id }=> html!(<TaskPage task_id={id.clone()} />),
        Route::History => html!(<HistoryPage />),
    };

    let header_component = if &Route::Login == &props.route {
        html!()
    } else {
        html!(<Header is_logged_in={*is_logged_in} logout={logout} />)
    };

    html!(
        <>
            { header_component }      
            <main class={style}>
                { body }
            </main>
        </>
    )
}

#[function_component(Home)]
fn home() -> Html {
    let navigator = use_navigator().unwrap();
    let state = use_reducer(ModelState::default);
    let token: UseStateHandle<Option<String>> = use_state(|| LocalStorage::get("access_token").ok());
    let client = Arc::new(Mutex::new(client_with_token((*token).clone())));

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
        let navigator = navigator.clone();

        let prompt = state.prompt.clone();

        Callback::from(move |_| {
            let client = client.clone();
            let state = state.clone();
            let navigator = navigator.clone();

            let prompt = prompt.clone();

            spawn_local(async move {
                let mut client = client.lock().unwrap();
                let res = client.create_task(CreateTaskRequest {
                    prompt,
                    iterations: 20,
                }).await.unwrap().into_inner();
                navigator.push(&Route::Task {
                    id: res.id.unwrap().id,
                });
            });
        })
    };

    html!(
        <div>
            <h1>{"image generation"}</h1>
            <input onchange={on_prompt_change} value={state.prompt.clone()} placeholder={"prompt"}/>
            <button onclick={run_inference}>{"run model"}</button>
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

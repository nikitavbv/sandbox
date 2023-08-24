use {
    std::{sync::{Arc, Mutex}, rc::Rc},
    yew::prelude::*,
    yew_router::prelude::*,
    serde::Deserialize,
    gloo_storage::{Storage, LocalStorage},
    web_sys::{EventTarget, HtmlInputElement},
    wasm_bindgen::JsCast,
    stylist::{style, yew::styled_component},
    wasm_bindgen_futures::spawn_local,
    rpc::{CreateTaskRequest, TaskParams, task_params::{Params, ImageGenerationParams as RpcImageGenerationParams}},
    crate::{
        utils::{client_with_token, Route},
        components::prompt_input::PromptInput,
    },
    self::{
        chat::ChatTaskCreation,
        reducer::{TaskCreationParams, ImageGenerationParams, TaskCreationParamsAction},
    },
};

mod chat;
mod reducer;

#[derive(Deserialize, Debug)]
struct HomeQuery {
    enable_chat: Option<bool>,
}

#[derive(Properties, PartialEq)]
pub struct ImageGenerationTaskCreationProps {
    params: ImageGenerationParams,
    params_dispatcher: UseReducerDispatcher<TaskCreationParams>,
    token: Option<String>,
}

#[styled_component(HomePage)]
pub fn home() -> Html {
    let params = use_reducer(TaskCreationParams::default);
    let params_dispatcher = params.dispatcher();

    let token: UseStateHandle<Option<String>> = use_state(|| LocalStorage::get("access_token").ok());

    let location: Option<HomeQuery> = use_location().map(|v| v.query().unwrap());
    let enable_chat = location.and_then(|v| v.enable_chat).unwrap_or(false);

    let page_style = style!(r#"
        margin: 0 auto;
        width: 600px;
    "#).unwrap();

    let task_type_switch = if enable_chat {
        let task_type_switch_style = style!(r#"
            margin-bottom: 20px;

            .selected {
                color: black;
                background-color: white;
            }

            div {
                display: inline-block;
                border: 1px solid white;
                border-radius: 10px;
                padding: 8px;
                margin-right: 8px;
                cursor: pointer;
                user-select: none;
            }
        "#).unwrap();

        let image_generation_class = if let TaskCreationParams::ImageGeneration(_) = &*params {
            "selected"
        } else {
            ""
        };

        let chat_class = if let TaskCreationParams::Chat(_) = &*params {
            "selected"
        } else {
            ""
        };

        html!(
            <div class={task_type_switch_style}>
                <div class={image_generation_class} onclick={
                    let params_dispatcher = params_dispatcher.clone();
                    move |_| params_dispatcher.dispatch(TaskCreationParamsAction::SwitchToImageGeneration)
                }>{"image generation"}</div>
                <div class={chat_class} onclick={
                    let params_dispatcher = params_dispatcher.clone();
                    move |_| params_dispatcher.dispatch(TaskCreationParamsAction::SwitchToChat)
                }>{"chat"}</div>
            </div>
        )
    } else {
        html!()
    };

    let task_creation = match &*params {
        TaskCreationParams::ImageGeneration(params) => html!(<ImageGenerationTaskCreation params={params.clone()} params_dispatcher={params_dispatcher} token={(*token).clone()} />),
        TaskCreationParams::Chat(params) => html!(<ChatTaskCreation params={params.clone()} params_dispatcher={params_dispatcher} token={(*token).clone()} />),
    };

    html!(
        <div class={page_style}>
            { task_type_switch }
            { task_creation }
        </div>
    )
}

#[styled_component(ImageGenerationTaskCreation)]
pub fn image_generation_task_creation(props: &ImageGenerationTaskCreationProps) -> Html {
    let navigator = use_navigator().unwrap();
    let params = props.params.clone();
    let params_dispatcher = props.params_dispatcher.clone();
    let client = Arc::new(Mutex::new(client_with_token((props.token).clone())));

    let on_prompt_change = {
        let params_dispatcher = params_dispatcher.clone();

        Callback::from(move |e: Event| {
            let target: Option<EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
            if let Some(input) = input {
                params_dispatcher.dispatch(TaskCreationParamsAction::UpdateImageGenerationPrompt(input.value()));
            }
        })
    };

    let run_inference = {
        let params = params.clone();
        let client = client.clone();
        let navigator = navigator.clone();

        let prompt = params.prompt.clone();

        Callback::from(move |_| {
            let client = client.clone();
            let navigator = navigator.clone();

            let prompt = prompt.clone();
            let params = params.clone();

            spawn_local(async move {
                let mut client = client.lock().unwrap();
                let res = client.create_task(CreateTaskRequest {
                    params: Some(TaskParams {
                        params: Some(Params::ImageGeneration(RpcImageGenerationParams {
                            iterations: 20,
                            number_of_images: params.number_of_images,
                            prompt,
                        })),
                    }),
                }).await.unwrap().into_inner();
                navigator.push(&Route::Task {
                    id: res.id.unwrap().id,
                });
            });
        })
    };

    let description_style = style!(r#"
        width: 100%;
        text-align: center;
        user-select: none;
        padding-bottom: 16px;
        display: block;
        font-size: 16pt;
        line-height: 24pt;
        color: white;
    "#).unwrap();

    let option_row_style = style!(r#"
        display: flex;
        margin-top: 16px;
        height: 32px;
    "#).unwrap();

    let option_name_style = style!(r#"
        flex: 1;
        line-height: 32px;
        user-select: none;
    "#).unwrap();

    let option_selector_style = style!(r#"
        display: flex;

        div {
            border: 1px solid white;
            border-right: 0px;
            user-select: none;
            width: 32px;
            height: 32px;
            line-height: 32px;
            text-align: center;
            cursor: pointer;
            transition: color 0.2s ease-out, background-color 0.2s ease-out;
        }

        div:first-child {
            border-radius: 3px 0 0 3px;
        }

        div:hover {
            background-color: white;
            color: black;
        }

        .selected {
            background-color: white;
            color: black;
        }

        input {
            outline: none;
            border-radius: 0 3px 3px 0;
            border: 1px solid white;
            width: 70px;
            padding: 0 8px;
            text-align: center;
        }
    "#).unwrap();

    let number_of_images_options = [1, 5, 10];

    let number_of_images_components = number_of_images_options
        .into_iter()
        .map(|v| html!(<div 
            class={if v == params.number_of_images && !params.number_of_images_custom { "selected" } else { "" }}
            onclick={
                let params_dispatcher = params_dispatcher.clone();
                move |_| { params_dispatcher.dispatch(TaskCreationParamsAction::SelectNumberOfImagesOption(v)) }
            }>{v.to_string()}</div>))
        .collect::<Vec<_>>();

    let on_number_of_images_custom_change = {
        let params_dispatcher = params_dispatcher.clone();
        
        move |e: Event| {
            let target: Option<EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
            if let Some(input) = input {
                params_dispatcher.dispatch(TaskCreationParamsAction::SetCustomNumberOfImages(input.value().parse().unwrap()));
            }
        }
    };

    html!(
        <>
            <span class={description_style}>{"Provide a text description of an image, and this app will generate it for you!"}</span>
            <PromptInput 
                value={params.prompt.clone()} 
                on_change={
                    let params_dispatcher = props.params_dispatcher.clone();
                    move |v| params_dispatcher.dispatch(TaskCreationParamsAction::UpdateImageGenerationPrompt(v))
                } 
                on_run_inference={run_inference} />
            <div class={option_row_style}>
                <div class={option_name_style}>{"number of images"}</div>
                <div class={option_selector_style}>
                    { number_of_images_components }
                    <input type="number" onchange={on_number_of_images_custom_change} value={if params.number_of_images_custom { Some(params.number_of_images.to_string()) } else { None }} />
                </div>
            </div>
        </>
    )
}

use {
    std::sync::{Arc, Mutex},
    yew::prelude::*,
    yew_router::prelude::*,
    tracing::info,
    stylist::{style, yew::styled_component},
    wasm_bindgen_futures::spawn_local,
    rpc::{CreateTaskRequest, TaskParams, task_params::{Params, ChatMessageGenerationParams}, AddChatUserMessageRequest},
    crate::{
        components::{prompt_input::PromptInput, model_highlight::ModelHighlight},
        utils::{client_with_token, Route},
    },
    super::reducer::{ChatParams, TaskCreationParams, TaskCreationParamsAction},
};

#[derive(Properties, PartialEq)]
pub struct ChatTaskCreationProps {
    pub params: ChatParams,
    pub params_dispatcher: UseReducerDispatcher<TaskCreationParams>,
    pub token: Option<String>,
}

#[styled_component(ChatTaskCreation)]
pub fn chat_task_creation(props: &ChatTaskCreationProps) -> Html {
    let navigator = use_navigator().unwrap();
    let params = props.params.clone();
    let client = Arc::new(Mutex::new(client_with_token((props.token).clone())));
    
    let start_chat = {
        let client = client.clone();
        let navigator = navigator.clone();

        let message = params.message.clone();

        Callback::from(move |_| {
            let client = client.clone();
            let navigator = navigator.clone();

            let message = message.clone();

            spawn_local(async move {
                let mut client = client.lock().unwrap();
                let res = client.create_task(CreateTaskRequest {
                    params: Some(TaskParams {
                        params: Some(Params::ChatMessageGeneration(ChatMessageGenerationParams {})),
                    }),
                }).await.unwrap().into_inner();

                let task_id = res.id.unwrap();

                client.add_chat_user_message(AddChatUserMessageRequest {
                    task_id: Some(task_id.clone()),
                    content: message,
                }).await.unwrap();

                navigator.push(&Route::Task {
                    id: task_id.id,
                });
            });
        })
    };

    html!(
        <>
            <ModelHighlight>{"Enter your message to chat with LLM-powered assistant!"}</ModelHighlight>
            <PromptInput 
                description={"message, for example: what is chocolate made of?"}
                action_name={"chat"}
                action_button_width={100}
                value={params.message.clone()}
                on_change={
                    let params_dispatcher = props.params_dispatcher.clone();
                    move |v| params_dispatcher.dispatch(TaskCreationParamsAction::UpdateChatMessage(v))
                }
                on_run_inference={start_chat} />
        </>
    )
}
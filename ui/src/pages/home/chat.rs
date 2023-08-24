use {
    yew::prelude::*,
    tracing::info,
    stylist::{style, yew::styled_component},
    crate::components::prompt_input::PromptInput,
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
    let params = props.params.clone();
    
    let start_chat = {
        Callback::from(move |_| {
            info!("not implemented yet");
        })
    };

    html!(
        <>
            <PromptInput 
                value={params.message.clone()}
                on_change={
                    let params_dispatcher = props.params_dispatcher.clone();
                    move |v| params_dispatcher.dispatch(TaskCreationParamsAction::UpdateChatMessage(v))
                }
                on_run_inference={start_chat} />
        </>
    )
}
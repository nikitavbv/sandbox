use {
    yew::prelude::*,
    tracing::info,
    stylist::{style, yew::styled_component},
    crate::components::{prompt_input::PromptInput, model_highlight::ModelHighlight},
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
            <ModelHighlight>{"Enter your message to chat with LLM-powered assistant!"}</ModelHighlight>
            <PromptInput 
                description={"message, for example: what is chocolate made of?"}
                action_name={"chat"}
                value={params.message.clone()}
                on_change={
                    let params_dispatcher = props.params_dispatcher.clone();
                    move |v| params_dispatcher.dispatch(TaskCreationParamsAction::UpdateChatMessage(v))
                }
                on_run_inference={start_chat} />
        </>
    )
}
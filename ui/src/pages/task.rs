use {
    std::sync::{Arc, Mutex},
    yew::prelude::*,
    tracing::info,
    wasm_bindgen_futures::spawn_local,
    rpc::{TaskId, TaskStatus},
    crate::utils::client,
};

#[derive(Properties, PartialEq)]
pub struct TaskPageProps {
    pub task_id: String,
}

#[function_component(TaskPage)]
pub fn task_page(props: &TaskPageProps) -> Html {
    let client = Arc::new(Mutex::new(client()));
    let state = use_state(|| None::<TaskStatus>);
    let state_setter = state.setter();

    use_effect_with_deps(move |id| {
        let client = client.clone();
        let id = id.clone();
        let state_setter = state_setter.clone();

        spawn_local(async move {
            let mut client = client.lock().unwrap();
        
            let status = client.get_task_status(TaskId {
                id,
            }).await.unwrap().into_inner();
            state_setter.set(Some(status));
        });
        
        || ()
    }, (props.task_id.clone()));

    let rendered = match &*state {
        None => html!(<div>{"loading task status..."}</div>),
        Some(v) => html!(<div>{"loaded task status..."}</div>),
    };

    html!(
        <div>
            <h1>{"image generation"}</h1>
            { rendered }
        </div>
    )
}
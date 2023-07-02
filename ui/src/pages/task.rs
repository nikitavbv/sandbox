use {
    std::sync::{Arc, Mutex},
    yew::prelude::*,
    yew_router::prelude::*,
    tracing::info,
    wasm_bindgen_futures::spawn_local,
    rpc::{TaskId, Task, GetTaskRequest},
    crate::utils::{client, Route},
};

#[derive(Properties, PartialEq)]
pub struct TaskPageProps {
    pub task_id: String,
}

#[function_component(TaskPage)]
pub fn task_page(props: &TaskPageProps) -> Html {
    let navigator = use_navigator().unwrap();

    let client = Arc::new(Mutex::new(client()));
    let state = use_state(|| None::<Task>);
    let state_setter = state.setter();

    use_effect_with_deps(move |id| {
        let client = client.clone();
        let id = id.clone();
        let state_setter = state_setter.clone();

        spawn_local(async move {
            let mut client = client.lock().unwrap();
        
            let res = client.get_task(GetTaskRequest {
                id: Some(TaskId {
                    id,
                }),
            }).await.unwrap().into_inner();
            state_setter.set(Some(res.task.unwrap()));
        });
        
        || ()
    }, props.task_id.clone());

    let return_home = Callback::from(move |_| {
        navigator.push(&Route::Home);
    });

    let rendered = match &*state {
        None => html!(<div>{"loading task status..."}</div>),
        Some(v) => if let rpc::task::Status::FinishedDetails(finished) = v.status.as_ref().unwrap() {
            html!(
                <div>
                    <img src={format!("data:image/png;base64, {}", base64::encode(&finished.image))} style={"display: block;"} />
                    <p style="font-style: italic;">{ v.prompt.clone() }</p>
                </div>
            )
        } else {
            html!(<div>{"task is not complete yet"}</div>)
        },
    };

    html!(
        <div>
            <button onclick={return_home}>{"home"}</button>
            <h1>{"image generation"}</h1>
            { rendered }
        </div>
    )
}
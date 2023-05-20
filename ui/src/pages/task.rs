use {
    std::sync::{Arc, Mutex},
    yew::prelude::*,
    yew_router::prelude::*,
    tracing::info,
    wasm_bindgen_futures::spawn_local,
    rpc::{TaskId, TaskStatus},
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
    }, props.task_id.clone());

    let return_home = Callback::from(move |_| {
        navigator.push(&Route::Home);
    });

    let rendered = match &*state {
        None => html!(<div>{"loading task status..."}</div>),
        Some(v) => if v.is_complete {
            html!(
                <div>
                    <img src={format!("data:image/png;base64, {}", base64::encode(v.image.as_ref().unwrap()))} style={"display: block;"} />
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
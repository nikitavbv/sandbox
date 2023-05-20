use {
    std::sync::{Arc, Mutex},
    yew::prelude::*,
    yew_router::prelude::*,
    wasm_bindgen_futures::spawn_local,
    rpc::{TaskStatus, HistoryRequest},
    crate::utils::{Route, client},
};

#[derive(Properties, Eq, PartialEq)]
pub struct HistoryEntryProps {
    id: String,
    prompt: String,
    image: Option<Vec<u8>>,
}

#[function_component(HistoryPage)]
pub fn history_page() -> Html {
    let navigator = use_navigator().unwrap();
    
    let return_home = Callback::from(move |_| {
        navigator.push(&Route::Home);
    });

    let client = Arc::new(Mutex::new(client()));
    let state = use_state(|| None::<Vec<TaskStatus>>);
    let state_setter = state.setter();

    use_effect(move || {
        let client = client.clone();
        let state_setter = state_setter.clone();

        spawn_local(async move {
            let mut client = client.lock().unwrap();
            let tasks = client.get_task_history(HistoryRequest {}).await.unwrap().into_inner();
            state_setter.set(Some(tasks.tasks));
        })
    });

    let tasks: Vec<_> = state.iter()
        .flat_map(|v| v.iter())
        .map(|v| html!(<HistoryEntry id={v.id.clone()} prompt={v.prompt.clone()} image={v.image.clone()} />))
        .collect();

    html!(
        <div>
            <button onclick={return_home}>{"home"}</button>
            <h1>{"history"}</h1>
            { tasks }
        </div>
    )
}

#[function_component(HistoryEntry)]
pub fn history_entry(props: &HistoryEntryProps) -> Html {
    let navigator = use_navigator().unwrap();

    let image = match props.image.as_ref() {
        Some(v) => html!(<img src={format!("data:image/png;base64, {}", base64::encode(v))} style={"width: 128px; height: 128px;"} />),
        None => html!(<span style={{"width: 128px; height: 128px; display: inline-block; vertical-align: middle; text-align: center;"}}>{"in progress..."}</span>),
    };

    let open_task = {
        let id = props.id.clone();
        let navigator = navigator.clone();

        Callback::from(move |_| {
            navigator.push(&Route::Task { id: id.clone() });
        })
    };

    html!(
        <div onclick={open_task} style={{"cursor: pointer;"}}>
            <span style={{"width: 128px; height: 128px; display: inline-block; vertical-align: middle; text-align: center;"}}>{props.prompt.clone()}</span>
            { image }
        </div>
    )
}
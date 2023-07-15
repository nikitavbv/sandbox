use {
    std::sync::{Arc, Mutex},
    yew::prelude::*,
    yew_router::prelude::*,
    wasm_bindgen_futures::spawn_local,
    rpc::{self, Task, GetAllTasksRequest},
    crate::utils::{Route, client},
};

#[derive(Properties, Eq, PartialEq)]
pub struct HistoryEntryProps {
    id: String,
    prompt: String,
    finished: bool,
}

#[function_component(HistoryPage)]
pub fn history_page() -> Html {
    let navigator = use_navigator().unwrap();
    
    let client = Arc::new(Mutex::new(client()));
    let state = use_state(|| None::<Vec<Task>>);
    let state_setter = state.setter();

    use_effect_with_deps(move |_| {
        let client = client.clone();
        let state_setter = state_setter.clone();

        spawn_local(async move {
            let mut client = client.lock().unwrap();
            let tasks = client.get_all_tasks(GetAllTasksRequest {}).await.unwrap().into_inner();
            state_setter.set(Some(tasks.tasks));
        })
    }, [None::<String>]);

    let tasks: Vec<_> = state.iter()
        .flat_map(|v| v.iter())
        .map(|v| html!(<HistoryEntry 
            id={v.id.as_ref().unwrap().id.clone()} 
            prompt={v.prompt.clone()}
            finished={is_finished(v)} />
        ))
        .collect();

    html!(
        <div>
            <h1>{"history"}</h1>
            { tasks }
        </div>
    )
}

#[function_component(HistoryEntry)]
pub fn history_entry(props: &HistoryEntryProps) -> Html {
    let navigator = use_navigator().unwrap();

    let image = if props.finished {
        html!(<img src={format!("/v1/storage/{}", props.id)} style={"width: 128px; height: 128px;"} />)
    } else {
        html!(<span style={{"width: 128px; height: 128px; display: inline-block; vertical-align: middle; text-align: center;"}}>{"in progress..."}</span>)
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

fn is_finished(task: &Task) -> bool {
    if let rpc::task::Status::FinishedDetails(_) = task.status.as_ref().unwrap() {
        return true;
    }

    false
}
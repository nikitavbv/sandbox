use {
    std::sync::{Arc, Mutex},
    yew::prelude::*,
    yew_router::prelude::*,
    wasm_bindgen_futures::spawn_local,
    stylist::{style, yew::styled_component},
    rpc::{self, Task, GetAllTasksRequest},
    crate::utils::{Route, client},
};

#[derive(Properties, Eq, PartialEq)]
pub struct HistoryEntryProps {
    id: String,
    prompt: String,
    finished: bool,
}

#[styled_component(HistoryPage)]
pub fn history_page() -> Html {
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

    let header_style = style!(r#"
        text-align: center;
        font-size: 16pt;
        margin-bottom: 16px;
    "#).unwrap();

    html!(
        <div>
            <h1 class={header_style}>{"All tasks"}</h1>
            { tasks }
        </div>
    )
}

#[styled_component(HistoryEntry)]
pub fn history_entry(props: &HistoryEntryProps) -> Html {
    let navigator = use_navigator().unwrap();
    
    let entry_style = style!(r#"
        width: 512px;
        margin: 0 auto;
        border: 1px solid #CED0CE;
        border-radius: 5px;
        margin-top: 20px;
        display: flex;
    "#).unwrap();

    let label_style = style!(r#"
        display: inline-block;
        margin: auto 0;
        padding: 0 20px;
    "#).unwrap();

    let image_style = style!(r#"
        width: 128px;
        height: 128px;
        background-color: #CED0CE;

        img {
            top: 0;
            left: 0;
            width: 100%;
            height: 100%;
        }
    "#).unwrap();

    let image = if props.finished {
        html!(
            <div class={image_style}>
                <img src={format!("/v1/storage/{}", props.id)} />
            </div>
        )
    } else {
        html!(<span style={{"width: 128px; height: 128px; display: inline-block; vertical-align: middle; text-align: center;"}}>{"in progress..."}</span>)
    };

    /*let open_task = {
        let id = props.id.clone();
        let navigator = navigator.clone();

        Callback::from(move |_| {
            navigator.push(&Route::Task { id: id.clone() });
        })
    };*/

    html!(
        <div class={entry_style}>
            { image }
            <span class={label_style}>{props.prompt.clone()}</span>
        </div>
    )
}

fn is_finished(task: &Task) -> bool {
    if let rpc::task::Status::FinishedDetails(_) = task.status.as_ref().unwrap() {
        return true;
    }

    false
}
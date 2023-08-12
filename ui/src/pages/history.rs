use {
    std::{sync::{Arc, Mutex}, time::Duration},
    tracing::info,
    yew::prelude::*,
    yew_router::prelude::*,
    wasm_bindgen_futures::spawn_local,
    stylist::{style, yew::styled_component},
    timeago::Formatter,
    tonic::Code,
    rpc::{self, Task, GetAllTasksRequest},
    crate::utils::{Route, client},
};

#[derive(Properties, Eq, PartialEq)]
pub struct HistoryEntryProps {
    id: String,
    prompt: String,
    finished: bool,
    time_since: Duration,
    cover_asset_id: Option<String>,
}

#[styled_component(HistoryPage)]
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
            let tasks = match client.get_all_tasks(GetAllTasksRequest {}).await {
                Ok(v) => v.into_inner(),
                Err(err) => match err.code() {
                    Code::Unauthenticated => {
                        navigator.push(&Route::Login);
                        return;
                    },
                    other => panic!("error while getting all tasks: {:?}", other),
                },
            };
            state_setter.set(Some(tasks.tasks));
        })
    }, [None::<String>]);

    let loading_style = style!(r#"
        text-align: center;
        font-size: 14pt;
        margin: 0 auto;
        display: block;
    "#).unwrap();

    let tasks = if state.is_some() {
        let tasks: Vec<_> = state.iter()
            .flat_map(|v| v.iter())
            .map(|v| html!(<HistoryEntry 
                id={v.id.as_ref().unwrap().id.clone()} 
                prompt={v.params.as_ref().unwrap().prompt.clone()}
                finished={is_finished(v)}
                time_since={Duration::from_secs(web_time::SystemTime::now().duration_since(web_time::UNIX_EPOCH).unwrap().as_secs() - v.created_at.as_ref().unwrap().seconds as u64)}
                cover_asset_id={v.assets.get(0).map(|v| v.id.clone())} />
            ))
            .collect();

        html!(<>{ tasks }</>)
    } else {
        html!(<span class={loading_style}>{"Loading..."}</span>)
    };

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
        flex: 1;
        text-align: center;
    "#).unwrap();

    let image_style = style!(r#"
        width: 128px;
        height: 128px;
        background-color: #CED0CE;
        position: relative;

        span {
            position: absolute;
            display: block;
            left: 0;
            right: 0;
            line-height: 128px;
            text-align: center;
        }

        img {
            position: absolute;
            top: 0;
            left: 0;
        }

        img {
            top: 0;
            left: 0;
            width: 100%;
            height: 100%;
        }
    "#).unwrap();

    let in_progress_style = style!(r#"
        width: 128px; 
        height: 128px; 
        display: inline-block; 
        vertical-align: middle; 
        text-align: center;
        border-right: 1px solid #CED0CE;
        user-select: none;
        line-height: 128px;
    "#).unwrap();

    let controls_style = style!(r#"
        width: 128px;
        height: 128px;
        display: flex;
        align-items: center;

        button {
            margin: auto;
            display: block;
            padding: 8px 14px;
            font-size: 12pt;
            background-color: #5695DC;
            color: white;
            border: 2px solid #5695DC;
            border-radius: 4px;
            cursor: pointer;
            user-select: none;
            transition:
                color 0.2s ease-out, 
                background-color 0.2s ease-out;
        }

        button:hover {
            background-color: #F6F4F3;
            color: #5695DC;
        }
    "#).unwrap();

    let task_timestamp_style = style!(r#"
        display: block;
        margin-top: 12px;
        color: #CED0CE;
        font-size: 10pt;
    "#).unwrap();

    let image = if let Some(asset_id) = props.cover_asset_id.as_ref() {
        html!(
            <div class={image_style}>
                <span>{"loading..."}</span>
                <img src={format!("/v1/storage/{}", asset_id)} />
            </div>
        )
    } else {
        html!(<span class={in_progress_style}>{"in progress..."}</span>)
    };

    let open_task = {
        let id = props.id.clone();
        let navigator = navigator.clone();

        Callback::from(move |_| {
            navigator.push(&Route::Task { id: id.clone() });
        })
    };

    html!(
        <div class={entry_style}>
            { image }
            <div class={label_style}>
                <span>{props.prompt.clone()}</span>
                <span class={task_timestamp_style}>{Formatter::new().convert(props.time_since)}</span>
            </div>
            <div class={controls_style}>
                <button onclick={open_task}>{"open"}</button>
            </div>
        </div>
    )
}

fn is_finished(task: &Task) -> bool {
    if let rpc::task::Status::FinishedDetails(_) = task.status.as_ref().unwrap() {
        return true;
    }

    false
}
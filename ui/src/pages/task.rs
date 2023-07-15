use {
    std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}},
    tracing::info,
    yew::prelude::*,
    yew_router::prelude::*,
    wasm_bindgen_futures::spawn_local,
    gloo_timers::callback::Interval,
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

    {
        let client = client.clone();
        let state_setter = state_setter.clone();

        use_effect_with_deps(move |id| {
            let client = client.clone();
            let id = id.clone();
            let state_setter = state_setter.clone();

            {
                let id = id.clone();
                let client = client.clone();
                let state_setter = state_setter.clone();

                spawn_local(async move {
                    let mut client = client.lock().unwrap();
                
                    let res = client.get_task(GetTaskRequest {
                        id: Some(TaskId {
                            id,
                        }),
                    }).await.unwrap().into_inner();

                    state_setter.set(res.task);
                });
            }
            
            || {}
        }, props.task_id.clone());
    }

    use_effect_with_deps(move |(task_id, status)| {
        let mut interval = None;            
        
        if status.is_none() {
            info!("no info about task yet");
        } else if let Some(rpc::task::Status::FinishedDetails(_)) = status {
            info!("task is finished");
        } else {
            info!("task is not finished yet");
            
            let refresh_in_progress = Arc::new(AtomicBool::new(false));
            let id = task_id.clone();
            let state_setter = state_setter.clone();

            interval = Some(Interval::new(1000, move || {
                let id = id.clone();
                let client = client.clone();

                if refresh_in_progress.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                    let refresh_in_progress = refresh_in_progress.clone();
                    let state_setter = state_setter.clone();

                    spawn_local(async move {
                        let mut client = client.lock().unwrap();

                        let res = client.get_task(GetTaskRequest {
                            id: Some(TaskId { id }),
                        }).await.unwrap().into_inner();
                        
                        state_setter.set(res.task);

                        refresh_in_progress.store(false, Ordering::SeqCst);
                    });
                }
            }));
        }

        move || {
            if let Some(interval) = interval {
                interval.cancel();
            }
        }
    }, (props.task_id.clone(), state.as_ref().and_then(|v| v.status.clone())));

    let rendered = match &*state {
        None => html!(<div>{"loading task status..."}</div>),
        Some(v) => {
            match &v.status {
                Some(rpc::task::Status::PendingDetails(_)) => html!(<div>{"waiting for task to be picked by worker..."}</div>),
                Some(rpc::task::Status::InProgressDetails(in_progress)) => html!(<div>{format!("task in progress: {}/{}", in_progress.current_step, in_progress.total_steps)}</div>),
                Some(rpc::task::Status::FinishedDetails(_)) => html!(
                    <div>
                        <img src={format!("/v1/storage/{}", v.id.as_ref().unwrap().id)} style={"display: block;"} />
                        <p style="font-style: italic;">{ v.prompt.clone() }</p>
                    </div>
                ),
                None => html!(<div>{"refreshing task state..."}</div>),
            }
        }
    };

    html!(
        <div>
            { rendered }
        </div>
    )
}

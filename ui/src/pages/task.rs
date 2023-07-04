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
        let state = state.clone();

        use_effect_with_deps(move |id| {
            let client = client.clone();
            let id = id.clone();
            let state_setter = state_setter.clone();
            let interval = Arc::new(Mutex::new(None::<Interval>));

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
            
            /*let new_interval = {
                let interval = interval.clone();
                let refresh_in_progress = Arc::new(AtomicBool::new(false));
                let id = Some(TaskId { id });

                Interval::new(100, move || {
                    let state = match &*state {
                        Some(v) => v,
                        None => {
                            info!("no informating about the task has been received yet");
                            return;
                        }
                    };

                    // TODO: implement status refresh
                    if refresh_in_progress.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                        info!("sending request to server");
                    
                        let client = client.clone();
                        let id = id.clone();
                        let refresh_in_progress = refresh_in_progress.clone();
                        let state_setter = state_setter.clone();

                        spawn_local(async move {
                            let mut client = client.lock().unwrap();

                            let res = client.get_task(GetTaskRequest {
                                id,
                            }).await.unwrap().into_inner();

                            state_setter.set(res.task);
                        
                            refresh_in_progress.store(false, Ordering::SeqCst);
                        });
                    }
                    
                    if let rpc::task::Status::FinishedDetails(_) = state.status.as_ref().unwrap() {
                        info!("task is ready, there is no reason to refresh it all the time");
                        interval.lock().unwrap().take().unwrap().cancel();
                        return;
                    }

                    info!("waiting for task");
                })
            };
            *interval.lock().unwrap() = Some(new_interval);

            move || {
                let mut interval = interval.lock().unwrap();
                
                if interval.is_some() {
                    interval.take().unwrap().cancel();
                }
            }*/

            || {}
        }, props.task_id.clone());
    }

    use_effect_with_deps(|(task_id, status)| {
        if status.is_none() {
            info!("no info about task yet");
            return;
        }

        if let Some(rpc::task::Status::FinishedDetails(_)) = status {
            info!("task is finished");
            return;
        }

        info!("task is not finished yet");

        let id = task_id.clone();
        spawn_local(async move {
            let mut client = client.lock().unwrap();

            let res = client.get_task(GetTaskRequest {
                id: Some(TaskId { id }),
            }).await.unwrap();

            info!("res: {:?}", res);
        });
    }, (props.task_id.clone(), state.as_ref().and_then(|v| v.status.clone())));

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

fn is_finished_task(task: &Task) -> bool {
    match task.status.as_ref().unwrap() {
        rpc::task::Status::FinishedDetails(_) => true,
        _ => false,
    }
}
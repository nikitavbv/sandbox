use {
    std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}},
    tracing::info,
    yew::prelude::*,
    yew_router::prelude::*,
    wasm_bindgen_futures::spawn_local,
    gloo_timers::callback::Interval,
    stylist::{style, yew::styled_component},
    rpc::{TaskId, Task, GetTaskRequest},
    crate::utils::{client, Route, MultiClass},
};

#[derive(Properties, PartialEq)]
pub struct TaskPageProps {
    pub task_id: String,
}

#[styled_component(TaskPage)]
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

    let loading_style = style!(r#"
        text-align: center;
        font-size: 20pt;
        font-weight: 100;
        padding-top: 200px;
    "#).unwrap();

    let image_style = style!(r#"
        display: block;
        margin: 0 auto;
    "#).unwrap();

    let image_placeholder_style = style!(r#"
        width: 512px;
        height: 512px;
        border: 1px solid #CED0CE;
        color: #CED0CE;
        text-align: center;
        line-height: 512px;
        user-select: none;
    "#).unwrap();

    let status_style = style!(r#"
        width: 512px;
        margin: 0 auto;

        span {
            display: block;
            padding: 12px 0;
            text-align: center;
            user-select: none;
        }
    "#).unwrap();

    let rendered = match &*state {
        None => html!(<div class={loading_style}>{"loading task status..."}</div>),
        Some(v) => {
            if v.status.is_none() {
                return html!(<div class={loading_style}>{"loading task status..."}</div>);
            }

            let image = match v.status.as_ref().unwrap() {
                rpc::task::Status::FinishedDetails(_) => html!(<img src={format!("/v1/storage/{}", v.id.as_ref().unwrap().id)} class={image_style} />),
                _ => html!(<div class={MultiClass::new().with(&image_style).with(&image_placeholder_style)}>{ &v.prompt }</div>),
            };

            let status = match v.status.as_ref().unwrap() {
                rpc::task::Status::PendingDetails(_) => html!(<>
                    <span>{"waiting for image generation task to be picked by worker..."}</span>
                    <span>{"this normally takes a few seconds, but may be longer if multiple tasks are in queue"}</span>
                </>),
                _ => html!(),
            };

            /*match v.status.as_ref().unwrap() {
                rpc::task::Status::PendingDetails(_) => html!(<div>{"waiting for task to be picked by worker..."}</div>),
                rpc::task::Status::InProgressDetails(in_progress) => html!(<div>{format!("task in progress: {}/{}", in_progress.current_step, in_progress.total_steps)}</div>),
                rpc::task::Status::FinishedDetails(_) => html!(
                    <div>
                        <img src={format!("/v1/storage/{}", v.id.as_ref().unwrap().id)} style={"display: block;"} />
                        <p style="font-style: italic;">{ v.prompt.clone() }</p>
                    </div>
                ),
            }*/

            html!(
                <div>
                    { image }
                    <div class={status_style}>{ status }</div>
                </div>
            )
        }
    };

    html!(
        <div>
            { rendered }
        </div>
    )
}

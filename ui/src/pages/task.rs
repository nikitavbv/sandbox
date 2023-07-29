use {
    std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}},
    tracing::info,
    yew::prelude::*,
    yew_router::prelude::*,
    wasm_bindgen_futures::spawn_local,
    gloo_timers::callback::Interval,
    stylist::{style, yew::styled_component},
    rpc::{TaskId, Task, TaskParams, GetTaskRequest, InProgressTaskDetails},
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
        padding-top: 240px;
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
        margin: 0 auto 20px auto;

        span {
            display: block;
            padding: 0;
            text-align: center;
            user-select: none;
            line-height: 20pt;
        }
    "#).unwrap();

    let progress_bar_style = style!(r#"
        width: 512px;
        height: 24px;
        border: 1px solid white;
        margin-top: 20px;
    "#).unwrap();
    
    let progress_bar_bar_style = style!(r#"
        background-color: white;
        height: 100%;

        transition: width 0.4s linear;
    "#).unwrap();

    let progress_bar_text_style = style!(r#"
        color: white;
        mix-blend-mode: difference;
        width: 512px;
        text-align: center;
        line-height: 24px;
        vertical-align: middle;
    "#).unwrap();

    let prompt_info_style = style!(r#"
        font-style: italic;
        with: 512px;
        text-algin: center;
    "#).unwrap();

    let rendered = match &*state {
        None => html!(<div class={loading_style}>{"loading task status..."}</div>),
        Some(task) => {
            if task.status.is_none() {
                return html!(<div class={loading_style}>{"loading task status..."}</div>);
            }

            let image = match task.status.as_ref().unwrap() {
                rpc::task::Status::FinishedDetails(_) => html!(<img src={format!("/v1/storage/{}", task.assets.get(0).unwrap().id)} class={image_style} />),
                _ => html!(<div class={MultiClass::new().with(&image_style).with(&image_placeholder_style)}>{ &task.prompt }</div>),
            };

            info!("task status: {:?}", task.status);

            let status = match task.status.as_ref().unwrap() {
                rpc::task::Status::PendingDetails(_) => html!(<>
                    <span>{"waiting for image generation task to be picked by worker..."}</span>
                    <span>{"this normally takes a few seconds, but may be longer if multiple tasks are in queue"}</span>
                </>),
                rpc::task::Status::InProgressDetails(in_progress) => html!(<>
                    <div class={progress_bar_style}>
                        <div class={progress_bar_bar_style} style={format!("width: {}%;", calculate_progress(task.params.as_ref(), in_progress) * 100.0)}>
                            <div class={progress_bar_text_style}>{progress_text(task.params.as_ref(), in_progress)}</div>
                        </div>
                    </div>
                </>),
                rpc::task::Status::FinishedDetails(_) => html!(<>
                    <span class={prompt_info_style}>{ &task.prompt }</span>
                </>),
            };

            html!(
                <div>
                    <div class={status_style}>{ status }</div>
                    { image }
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

fn calculate_progress(params: Option<&TaskParams>, in_progress_details: &InProgressTaskDetails) -> f32 {
    let steps_per_image = in_progress_details.total_steps as f32;
    let total_steps = steps_per_image * (params.map(|v| v.number_of_images).unwrap_or(1) as f32);

    (steps_per_image * in_progress_details.current_image as f32 + in_progress_details.current_step as f32) / total_steps
}

fn progress_text(params: Option<&TaskParams>, in_progress_details: &InProgressTaskDetails) -> String {
    let total_images = params.map(|v| v.number_of_images).unwrap_or(1);

    if total_images == 1 {
        format!("generating image: {}/{} steps", in_progress_details.current_step, in_progress_details.total_steps)
    } else {
        format!("generating image {} out of {}", in_progress_details.current_image + 1, total_images)
    }
}
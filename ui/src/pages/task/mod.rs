use {
    std::{sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}}, rc::Rc},
    tracing::info,
    yew::prelude::*,
    yew_router::prelude::*,
    wasm_bindgen_futures::spawn_local,
    gloo_timers::callback::Interval,
    stylist::{style, yew::styled_component},
    rpc::{TaskId, Task, TaskParams, GetTaskRequest, task_params::Params},
    crate::utils::{client, Route, MultiClass},
    self::image_generation::ImageGenerationTask,
};

mod image_generation;

#[derive(Properties, PartialEq)]
pub struct TaskPageProps {
    pub task_id: String,
}

#[derive(Clone)]
pub struct TaskState {
    task: Option<Task>,
}

pub enum TaskStateAction {
    LoadTask(Task),
}

impl Default for TaskState {
    fn default() -> Self {
        Self {
            task: None,
        }
    }
}

impl Reducible for TaskState {
    type Action = TaskStateAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            Self::Action::LoadTask(task) => Self {
                task: Some(task),
                ..(*self).clone()
            },
        }.into()
    }
}

#[styled_component(TaskPage)]
pub fn task_page(props: &TaskPageProps) -> Html {
    let navigator = use_navigator().unwrap();

    let client = Arc::new(Mutex::new(client()));
    let state = use_reducer(TaskState::default);
    let state_dispatcher = state.dispatcher();

    {
        let client = client.clone();
        let state_dispatcher = state_dispatcher.clone();

        use_effect_with_deps(move |id| {
            let client = client.clone();
            let id = id.clone();
            let state_dispatcher = state_dispatcher.clone();

            {
                let id = id.clone();
                let client = client.clone();
                let state_dispatcher = state_dispatcher.clone();

                spawn_local(async move {
                    let mut client = client.lock().unwrap();
                
                    let res = client.get_task(GetTaskRequest {
                        id: Some(TaskId {
                            id,
                        }),
                    }).await.unwrap().into_inner();

                    state_dispatcher.dispatch(TaskStateAction::LoadTask(res.task.unwrap()));
                });
            }
            
            || {}
        }, props.task_id.clone());
    }

    {
        let state_dispatcher = state_dispatcher.clone();

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
                let state_dispatcher = state_dispatcher.clone();
    
                interval = Some(Interval::new(1000, move || {
                    let id = id.clone();
                    let client = client.clone();
    
                    if refresh_in_progress.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                        let refresh_in_progress = refresh_in_progress.clone();
                        let state_dispatcher = state_dispatcher.clone();
    
                        spawn_local(async move {
                            let mut client = client.lock().unwrap();
    
                            let res = client.get_task(GetTaskRequest {
                                id: Some(TaskId { id }),
                            }).await.unwrap().into_inner();
                            
                            state_dispatcher.dispatch(TaskStateAction::LoadTask(res.task.unwrap()));
    
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
        }, (props.task_id.clone(), state.task.as_ref().and_then(|v| v.status.clone())));
    }

    let loading_style = style!(r#"
        text-align: center;
        font-size: 20pt;
        font-weight: 100;
        padding-top: 240px;
    "#).unwrap();

    let rendered = match &state.task {
        None => html!(<div class={loading_style}>{"loading task status..."}</div>),
        Some(task) => {
            if task.status.is_none() {
                return html!(<div class={loading_style}>{"loading task status..."}</div>);
            }

            match task.params.clone().unwrap().params.unwrap() {
                Params::ImageGeneration(v) => html!(<ImageGenerationTask status={task.status.clone().unwrap()} params={v} assets={task.assets.clone()} />),
                Params::ChatMessageGeneration(_) => html!(<span>{"chat tasks are not supported yet"}</span>),
            }
        }
    };

    html!(
        <div>
            { rendered }
        </div>
    )
}

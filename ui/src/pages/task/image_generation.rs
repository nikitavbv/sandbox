use {
    std::rc::Rc,
    yew::prelude::*,
    yew_router::prelude::*,
    stylist::{style, yew::styled_component},
    rpc::{task::Status, task_params::ImageGenerationParams, TaskAsset, InProgressTaskDetails},
    crate::utils::MultiClass,
};

#[derive(Properties, PartialEq)]
pub struct ImageGenerationTaskProps {
    pub status: Status,
    pub params: ImageGenerationParams,
    pub assets: Vec<TaskAsset>,
}

#[derive(Clone)]
pub struct TaskState {
    focused_asset: Option<String>,
}

pub enum TaskStateAction {
    FocusOnAsset(String),
}

impl Default for TaskState {
    fn default() -> Self {
        Self {
            focused_asset: None,
        }
    }
}

impl Reducible for TaskState {
    type Action = TaskStateAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            Self::Action::FocusOnAsset(asset_id) => Self {
                focused_asset: Some(asset_id),
                ..(*self).clone()
            },
        }.into()
    }
}

#[styled_component(ImageGenerationTask)]
pub fn image_generation_task(props: &ImageGenerationTaskProps) -> Html {
    let state = use_reducer(TaskState::default);
    let state_dispatcher = state.dispatcher();

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
        user-select: none;
    "#).unwrap();

    let prompt_info_style = style!(r#"
        font-style: italic;
        with: 512px;
        text-algin: center;
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

    let all_images_style = style!(r#"
        max-width: 600px;
        margin: 20px auto 0 auto;

        h1 {
            font-size: 20pt;
            text-align: center;
        }

        div {
            display: flex;
            flex-wrap: wrap;
            padding: 20px 0;
            justify-content: space-evenly;
        }

        div img {
            width: 80px;
            height: 80px;
            margin: 0 8px 8px 0;
            cursor: pointer;
        }
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

    let selected_asset_style = style!("outline: 2px solid white;").unwrap();

    let status = match &props.status {
        rpc::task::Status::PendingDetails(_) => html!(<>
            <span>{"waiting for image generation task to be picked by worker..."}</span>
            <span>{"this normally takes a few seconds, but may be longer if multiple tasks are in queue"}</span>
        </>),
        rpc::task::Status::InProgressDetails(in_progress) => {
            html!(<>
                <div class={progress_bar_style}>
                    <div class={progress_bar_bar_style} style={format!("width: {}%;", calculate_progress(&props.params, &in_progress) * 100.0)}>
                        <div class={progress_bar_text_style}>{progress_text(&props.params, &in_progress)}</div>
                    </div>
                </div>
            </>)
        },
        rpc::task::Status::FinishedDetails(_) => {
            let prompt = props.params.prompt.clone();
            
            html!(<>
                <span class={prompt_info_style}>{ prompt }</span>
            </>)
        },
    };

    let image = if !props.assets.is_empty() {
        let focused_asset_id = state.focused_asset.as_ref().cloned().unwrap_or(props.assets.get(0).unwrap().id.clone());
        html!(<img src={format!("/v1/storage/{}", focused_asset_id)} class={image_style} />)
    } else {
        html!(<div class={MultiClass::new().with(&image_style).with(&image_placeholder_style)}>{ &props.params.prompt }</div>)
    };

    let all_images = if props.assets.len() > 1 {
        let focused_asset_id = state.focused_asset.as_ref().cloned().unwrap_or(props.assets.get(0).unwrap().id.clone());

        let mut images = Vec::new();
        let selected_asset_style_class = selected_asset_style.get_class_name().to_owned();

        for asset in &props.assets {
            let state_dispatcher = state_dispatcher.clone();
            let asset_id = asset.id.clone();

            images.push(html!(<img 
                class={if focused_asset_id == asset_id { selected_asset_style_class.clone() } else { "".to_owned() }}
                draggable="false"
                src={format!("/v1/storage/{}", asset_id)} 
                onclick={Callback::from(move |_| state_dispatcher.dispatch(TaskStateAction::FocusOnAsset(asset_id.clone())))} />));
        }

        html!(
            <div class={all_images_style}>
                <h1>{"All images"}</h1>
                <div>{ images }</div>
            </div>
        )
    } else {
        html!()
    };

    html!(
        <div>
            <div class={status_style}>{ status }</div>
            { image }
            { all_images }
        </div>
    )
}

fn calculate_progress(params: &ImageGenerationParams, in_progress_details: &InProgressTaskDetails) -> f32 {
    let steps_per_image = in_progress_details.total_steps as f32;
    let total_steps = steps_per_image * (params.number_of_images as f32);

    (steps_per_image * in_progress_details.current_image as f32 + in_progress_details.current_step as f32) / total_steps
}

fn progress_text(params: &ImageGenerationParams, in_progress_details: &InProgressTaskDetails) -> String {
    let total_images = params.number_of_images;

    if total_images == 1 {
        format!("generating image: {}/{} steps", in_progress_details.current_step, in_progress_details.total_steps)
    } else {
        format!("generating image {} out of {}", in_progress_details.current_image + 1, total_images)
    }
}
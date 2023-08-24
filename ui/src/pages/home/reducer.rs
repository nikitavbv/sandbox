use {
    std::rc::Rc,
    yew::prelude::*,
};

#[derive(Clone)]
pub enum TaskCreationParams {
    ImageGeneration(ImageGenerationParams),
    Chat(ChatParams),
}

#[derive(Clone, PartialEq)]
pub struct ImageGenerationParams {
    pub prompt: String,
    pub number_of_images: u32,
    pub number_of_images_custom: bool,
}

#[derive(Clone, PartialEq)]
pub struct ChatParams {
    pub message: String,
}

pub enum TaskCreationParamsAction {
    SwitchToImageGeneration,
    UpdateImageGenerationPrompt(String),
    SelectNumberOfImagesOption(u32),
    SetCustomNumberOfImages(u32),

    SwitchToChat,
    UpdateChatMessage(String),
}

impl Default for TaskCreationParams {
    fn default() -> Self {
        Self::ImageGeneration(ImageGenerationParams::default())
    }
}

impl Default for ImageGenerationParams {
    fn default() -> Self {
        Self {
            prompt: "".to_owned(),
            number_of_images: 1,
            number_of_images_custom: false,
        }
    }
}

impl Default for ChatParams {
    fn default() -> Self {
        Self {
            message: "".to_owned(),
        }
    }
}

impl Reducible for TaskCreationParams {
    type Action = TaskCreationParamsAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            Self::Action::SwitchToImageGeneration => Self::ImageGeneration(ImageGenerationParams::default()),
            Self::Action::UpdateImageGenerationPrompt(prompt) => match &*self {
                Self::ImageGeneration(params) => Self::ImageGeneration(ImageGenerationParams {
                    prompt,
                    ..(params.clone())
                }),
                other => other.clone(),
            },
            Self::Action::SelectNumberOfImagesOption(number_of_images) => match &*self {
                Self::ImageGeneration(params) => Self::ImageGeneration(ImageGenerationParams {
                    number_of_images,
                    number_of_images_custom: false,
                    ..(params.clone())
                }),
                other => other.clone(),
            },
            Self::Action::SetCustomNumberOfImages(number_of_images) => match &*self {
                Self::ImageGeneration(params) => Self::ImageGeneration(ImageGenerationParams {
                    number_of_images,
                    number_of_images_custom: true,
                    ..(params.clone())
                }),
                other => other.clone(),
            },

            Self::Action::SwitchToChat => Self::Chat(ChatParams::default()),
            Self::Action::UpdateChatMessage(message) => match &*self {
                Self::Chat(params) => Self::Chat(ChatParams {
                    message,
                    ..(params.clone())
                }),
                other => other.clone(),
            }
        }.into()
    }
}

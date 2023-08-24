use {
    std::str::FromStr,
    ulid::Ulid,
    chrono::{DateTime, Utc},
};

pub struct Task {
    pub id: TaskId,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub params: TaskParams,
}

pub struct TaskId {
    id: String,
}

impl TaskId {
    pub fn new(id: String) -> Self {
        Self {
            id,
        }
    }

    pub fn as_str(&self) -> &str {
        &self.id
    }   
}

impl From<rpc::TaskId> for TaskId {
    fn from(value: rpc::TaskId) -> Self {
        Self::new(value.id)
    }
}

impl From<TaskId> for rpc::TaskId {
    fn from(value: TaskId) -> Self {
        Self { id: value.id }
    }
}

#[derive(Eq, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress {
        current_image: u32,
        current_step: u32,
        total_steps: u32,
    },
    Finished,
}

pub struct UserId {
    id: Ulid,
}

impl UserId {
    pub fn from_string(id: String) -> Self {
        Self {
            id: Ulid::from_str(&id).unwrap(),
        }
    }

    pub fn to_string(&self) -> String {
        self.id.to_string()
    }
}

pub struct AssetId {
    id: Ulid,
}

impl AssetId {
    pub fn from_string(id: String) -> Self {
        Self {
            id: Ulid::from_str(&id).unwrap(),
        }
    }

    pub fn to_string(&self) -> String {
        self.id.to_string()
    }
}

pub enum TaskParams {
    ImageGenerationParams {
        prompt: String,
        iterations: u32,
        number_of_images: u32,
    },
    ChatMessageGenerationParams {
    }
}

impl Default for TaskParams {
    fn default() -> Self {
        Self::ImageGenerationParams {
            prompt: "cute cat".to_owned(),
            iterations: 20,
            number_of_images: 1,
        }
    }
}

impl From<TaskParams> for rpc::task_params::Params {
    fn from(value: TaskParams) -> Self {
        match value {
            TaskParams::ImageGenerationParams { 
                prompt, 
                iterations, 
                number_of_images,
            } => rpc::task_params::Params::ImageGeneration(rpc::task_params::ImageGenerationParams {
                prompt,
                iterations,
                number_of_images,
            }),
            TaskParams::ChatMessageGenerationParams {} => rpc::task_params::Params::ChatMessageGeneration(rpc::task_params::ChatMessageGenerationParams {}),
        }
    }
}

pub struct ChatMessage {
    pub task_id: TaskId,
    pub message_id: MessageId,
    pub content: String,
    pub role: ChatMessageRole,
    pub index: u32,
}

pub struct MessageId {
    id: String,
}

impl MessageId {
    pub fn new(id: String) -> Self {
        Self {
            id,
        }
    }
}

impl From<MessageId> for rpc::MessageId {
    fn from(value: MessageId) -> Self {
        Self {
            id: value.id,
        }
    }
}

pub enum ChatMessageRole {
    System,
    User,
    Assistant,
}

impl From<ChatMessageRole> for rpc::ChatMessageRole {
    fn from(value: ChatMessageRole) -> Self {
        match value {
            ChatMessageRole::System => Self::System,
            ChatMessageRole::User => Self::User,
            ChatMessageRole::Assistant => Self::Assistant,
        }
    }
}
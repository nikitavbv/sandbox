use {
    std::str::FromStr,
    ulid::Ulid,
    chrono::{DateTime, Utc},
};

pub struct Task {
    pub id: TaskId,
    pub prompt: String,
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

pub struct TaskParams {
    pub iterations: u32,
    pub number_of_images: u32,
}

impl Default for TaskParams {
    fn default() -> Self {
        Self {
            iterations: 20,
            number_of_images: 1,
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

pub enum ChatMessageRole {
    System,
    User,
    Assistant,
}
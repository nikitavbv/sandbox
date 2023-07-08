pub struct Task {
    pub id: TaskId,
    pub prompt: String,
    pub status: TaskStatus,
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
        current_step: u32,
        total_steps: u32,
    },
    Finished,
}

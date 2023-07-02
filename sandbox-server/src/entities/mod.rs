pub struct Task {
    id: TaskId,
    prompt: String,
    status: TaskStatus,
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

#[derive(Eq, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress {
        current_step: u32,
        total_steps: Option<u32>,
    },
    Finished {
        image: Vec<u8>,
    },
}

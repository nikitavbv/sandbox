pub struct TaskId {
    id: String,
}

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
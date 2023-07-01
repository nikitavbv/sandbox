pub enum TaskStatus {
    Pending,
    Started,
    InProgress {
        current_step: u32,
        total_steps: u32,
    },
    Finished,
}
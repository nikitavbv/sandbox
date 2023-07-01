pub enum TaskStatus {
    Pending,
    InProgress {
        current_step: u32,
        total_steps: u32,
    },
    Finished {
        image: Vec<u8>,
    },
}
use crate::{
    entities::{TaskId, TaskStatus},
    state::{
        database::Database,
        storage::Storage,
    },
};

async fn update_task_status(database: &Database, storage: &Storage, id: TaskId, status: TaskStatus) {
    // TODO: implement this
}
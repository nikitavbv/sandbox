use crate::{
    entities::{TaskId, TaskStatus},
    state::{
        database::Database,
        storage::Storage,
    },
};

pub async fn update_task_status(database: &Database, id: TaskId, status: TaskStatus) {
    database.save_task_status(&id, &status).await;
}
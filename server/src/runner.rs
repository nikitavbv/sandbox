use {
    std::sync::Arc,
    tokio::sync::Mutex,
    actix_web::{Responder, web::Data},
    crate::models::SimpleMnistModel,
};

pub async fn simple_mnist_runner() -> impl Responder {
    "hello from simple mnist runner!\n"
}

pub async fn simple_mnist_trainer(model: Data<Arc<Mutex<SimpleMnistModel>>>) -> impl Responder {
    model.lock().await.train(); // this will block everything, but it is okay, because this will be refactored into a worker later.
    "done\n"
}
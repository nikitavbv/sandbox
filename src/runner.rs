use actix_web::{Responder, web::Path};

pub async fn runner(model_id: Path<String>) -> impl Responder {
    format!("hello from runner with model id {model_id}!\n")
}
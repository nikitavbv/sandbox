use actix_web::{Responder, web::Path};

pub async fn simple_mnist_runner() -> impl Responder {
    "hello from simple mnist runner!\n"
}
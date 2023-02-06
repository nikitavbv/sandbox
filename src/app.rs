use actix_web::{web::{self, ServiceConfig}, HttpRequest, Responder};

pub fn init(app: &mut ServiceConfig) {
    app.service(
        web::resource("/")
            .route(web::get().to(handler))
    );
}

async fn handler(req: HttpRequest) -> impl Responder {
    "hello from actix handler!"
}
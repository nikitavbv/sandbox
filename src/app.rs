use actix_web::{web::{self, ServiceConfig}, HttpRequest, Responder};

pub struct AiLabApp {
}

impl AiLabApp {
    pub fn new() -> Self {
        Self {
        }
    }

    pub fn init(&self, app: &mut ServiceConfig) {
        app.service(
            web::resource("/")
                .route(web::get().to(handler))
        );
    }
}

async fn handler(req: HttpRequest) -> impl Responder {
    "hello from actix handler!"
}
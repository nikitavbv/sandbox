use {
    actix_web::{web::{self, ServiceConfig}},
    crate::runner::runner,
};

pub struct AiLabApp {
}

impl AiLabApp {
    pub fn new() -> Self {
        Self {
        }
    }

    pub fn init(&self, app: &mut ServiceConfig) {
        app.service(
            web::scope("/api/v1")
                .route("/models/{model_id}/inference", web::get().to(runner))
        );
    }
}

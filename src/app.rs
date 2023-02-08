use {
    std::sync::Arc,
    actix_web::web::{self, ServiceConfig},
    tokio::sync::Mutex,
    crate::{
        runner::{simple_mnist_runner, simple_mnist_trainer},
        models::SimpleMnistModel,
    },
};

pub struct AiLabApp {
    model_simple_mnist: web::Data<Arc<Mutex<SimpleMnistModel>>>,
}

impl AiLabApp {
    pub fn new() -> Self {
        Self {
            model_simple_mnist: web::Data::new(Arc::new(Mutex::new(SimpleMnistModel::new()))),
        }
    }

    pub fn init(&self, app: &mut ServiceConfig) {
        app.service(
            web::scope("/api/v1")
                .app_data(self.model_simple_mnist.clone())
                .route("/models/simple-mnist/train", web::get().to(simple_mnist_trainer))
                .route("/models/simple-mnist/inference", web::get().to(simple_mnist_runner))
        );
    }
}

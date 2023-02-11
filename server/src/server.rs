use {
    std::io::Cursor,
    tonic::{transport::Server, Status, Request, Response},
    tokio::sync::Mutex,
    rpc::{
        ml_sandbox_service_server::{MlSandboxService, MlSandboxServiceServer},
        FILE_DESCRIPTOR_SET,
        RunSimpleModelRequest,
        RunSimpleModelResponse,
        TrainSimpleModelRequest,
        TrainSimpleModelResponse,
    },
    crate::models::SimpleMnistModel,
};

pub async fn run_server() {
    Server::builder()
        .accept_http1(true)
        .add_service(
            tonic_reflection::server::Builder::configure()
                .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
                .build()
                .unwrap()
        )
        .add_service(tonic_web::enable(MlSandboxServiceServer::new(MlSandboxServiceHandler::new())))
        .serve("0.0.0.0:8080".parse().unwrap())
        .await
        .unwrap();
}

struct MlSandboxServiceHandler {
    model: Mutex<SimpleMnistModel>,
}

impl MlSandboxServiceHandler {
    pub fn new() -> Self {
        Self {
            model: Mutex::new(SimpleMnistModel::new()),
        }
    }
}

#[tonic::async_trait]
impl MlSandboxService for MlSandboxServiceHandler {
    async fn run_simple_model(&self, req: Request<RunSimpleModelRequest>) -> Result<Response<RunSimpleModelResponse>, Status> {
        let image = req.into_inner().image;
        let image = image::io::Reader::new(Cursor::new(&image)).with_guessed_format().unwrap().decode().unwrap();

        let model = self.model.lock().await;
        
        Ok(Response::new(RunSimpleModelResponse {}))
    }

    async fn train_simple_model(&self, req: Request<TrainSimpleModelRequest>) -> Result<Response<TrainSimpleModelResponse>, Status> {
        let model = self.model.lock().await;

        model.train();
        
        Ok(Response::new(TrainSimpleModelResponse {}))
    }
}
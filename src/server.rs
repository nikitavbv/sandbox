use {
    std::{future::Future, sync::Arc},
    tracing::info,
    hyper::{
        service::{make_service_fn, service_fn},
        Server,
        Request,
        Response,
        Body,
    },
};

pub async fn run_http_server<T: Handler + Sync + Send + 'static>(request_handler: Arc<T>) {
    let service = make_service_fn(move |_| {
        let request_handler = request_handler.clone();

        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                handler(request_handler.clone(), req)
            }))
        }
    });

    let addr = ([0, 0, 0, 0], 8080).into();
    let server = Server::bind(&addr).serve(service);

    info!(addr=addr.to_string(), "http server listening");

    server.await.unwrap();
}

async fn handler<T: Handler + Sync + Send + 'static>(request_handler: Arc<T>, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    Ok(request_handler.handle(req).await)
}

#[async_trait::async_trait]
pub trait Handler {
    async fn handle(&self, req: Request<Body>) -> Response<Body>;
}

#[async_trait::async_trait]
impl <T: Fn(Request<Body>) -> Fut + Sync, Fut: Future<Output=Response<Body>> + Send> Handler for T {
    async fn handle(&self, req: Request<Body>) -> Response<Body> {
        self(req).await
    }
}
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
    futures_util::FutureExt,
};

pub async fn run_http_server<F, Fut>(request_handler: Arc<F>) where F: (Fn(Request<Body>) -> Fut) + Send + Sync + 'static, Fut: Future<Output=Response<Body>> + Send + 'static {
    let request_handler = request_handler.clone();
    
    let service = make_service_fn(move |_| {
        let request_handler = request_handler.clone();

        async move {
            let request_handler = request_handler.clone();

            Ok::<_, hyper::Error>(service_fn(move |req| {
                request_handler(req).map(|v| Ok(v) as Result<Response<Body>, hyper::Error>)
            }))
        }
    });

    let addr = ([0, 0, 0, 0], 8080).into();
    let server = Server::bind(&addr).serve(service);

    info!(addr=addr.to_string(), "http server listening");

    server.await.unwrap();
}

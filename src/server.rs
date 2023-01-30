use {
    tracing::info,
    hyper::{
        service::{make_service_fn, service_fn},
        Server,
        Request,
        Response,
        Body,
    },
};

pub async fn run_http_server() {
    let service = make_service_fn(move |_| {
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                handler(req)
            }))
        }
    });

    let addr = ([0, 0, 0, 0], 8080).into();
    let server = Server::bind(&addr).serve(service);

    info!(addr=addr.to_string(), "http server listening");

    server.await.unwrap();
}

async fn handler(_req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    Ok(Response::new(Body::from("Hello World!\n")))
}
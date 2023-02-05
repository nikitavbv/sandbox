use {
    hyper::{Method, Request, Response, Body},
    crate::server::Handler,
};

pub struct HttpMethodFilter<T: Handler> {
    method: Method,
    inner: T,
}

impl <T: Handler> HttpMethodFilter<T> {
    pub fn new(method: Method, inner: T) -> Self {
        Self {
            method,
            inner,
        }
    }
}

#[async_trait::async_trait]
impl <T: Handler + Send + Sync> Handler for HttpMethodFilter<T> {
    async fn handle(&self, req: Request<Body>) -> Response<Body> {
        if req.method() == self.method {
            self.inner.handle(req).await
        } else {
            unimplemented!() // skip
        }
    }
}

use crate::common::{Body, Request, Response};

pub trait UnaryService<ReqBody> {
    type Response;
    type Error: Into<crate::Error>;

    fn call(&mut self, request: Request<ReqBody>) -> Result<Response<Body>, crate::Error>;
}

use std::future::Future;
use crate::runtime::Service;
use crate::common::{Request, Response};

pub trait UnaryService<R> {
    type Response;
    type Future: Future<Output = Result<Response<Self::Response>, crate::Status>>;
    fn call(&mut self, request: Request<R>) -> Self::Future;
}

impl<T, M1, M2> UnaryService<M1> for T
where
    T: Service<Request<M1>, Response = Response<M2>, Error = crate::Status>,
{
    type Response = M2;
    type Future = T::Future;

    fn call(&mut self, request: Request<M1>) -> Self::Future {
        Service::call(self, request)
    }
}

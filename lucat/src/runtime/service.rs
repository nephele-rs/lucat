use std::future::Future;

pub trait Service<Request> {
    type Response;
    type Error;
    type Future: Future<Output = Result<Self::Response, Self::Error>>;
    fn call(&mut self, req: Request) -> Self::Future;
}

impl<'a, S, Request> Service<Request> for &'a mut S
where
    S: Service<Request> + 'a,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn call(&mut self, request: Request) -> S::Future {
        (**self).call(request)
    }
}

impl<S, Request> Service<Request> for Box<S>
where
    S: Service<Request> + ?Sized,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn call(&mut self, request: Request) -> S::Future {
        (**self).call(request)
    }
}

#[crate::async_trait]
pub trait InstantService<Request> {
    type Response;
    type Error;
    async fn call(&mut self, req: Request) -> Result<Self::Response, Self::Error>;
}

#[crate::async_trait]
pub trait SimpleInstantService {
    async fn call(&mut self, req: crate::Request<crate::Body>) -> Result<crate::Response<crate::Body>, crate::Error>;
}

use std::net::{TcpListener, TcpStream};
use http::{HeaderMap};
use cynthia::runtime::{self, Async};
use nephele::proto::h2::server;
use std::error::Error;
use crate::runtime::Service;
use crate::common::{self, Body, Request, Response};

#[derive(Clone)]
pub struct Server {
    id: i32,
}

impl Server {
    pub fn builder() -> Self {
        Server {
            id: 456,
        }
    }
}

impl Server {
    pub fn register<S>(&mut self, svc: S) -> Router<S> 
    where 
        S: Service<Request<Body>, Response = Response<Body>>
            + Clone 
            + Send 
            + Sync 
            + 'static,
        S::Future: Send + 'static,
        S::Error: Into<common::Error> + Send,
    {
        Router::new(self.clone(), svc)
    }

    pub async fn serve<S>(self, listener: Async<TcpListener>, route: Routes<S>) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        S: Service<Request<Body>, Response = Response<Body>>
            + Clone 
            + Send 
            + Sync 
            + 'static,
        S::Future: Send + 'static,
        <S as Service<Request<Body>>>::Error: Send + Sync + std::error::Error,
    {
        loop {
            let (stream, _peer_addr) = listener.accept().await?;
            let r = route.clone();
            runtime::spawn(r.handle(stream)).detach();
        }
    }
}

pub struct Router<S> {
    server: Server,
    routes: Routes<S>,
}

impl<S> Router<S> {
    fn new(server: Server, svc: S) -> Self
    where
        S: Service<Request<Body>, Response = Response<Body>>
            + Clone 
            + 'static,
        S::Future: Send + 'static,
        S::Error: Into<common::Error> + Send,
    {
        Router {
            server: server,
            routes: Routes::new(svc)
        }
    }

    pub async fn serve(self, listener: Async<TcpListener>) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        S: Service<Request<Body>, Response = Response<Body>>
            + Clone 
            + Send 
            + Sync 
            + 'static,
        S::Future: Send + 'static,
        <S as Service<Request<Body>>>::Error: Send + Sync + std::error::Error,
    {
        self.server.serve(listener, self.routes).await
    }
}

#[derive(Default, Clone)]
pub struct Routes<B> {
    inner: B,
}

impl<S> Routes<S> {
    fn new(inner: S) -> Routes<S> {
        Routes {
            inner: inner
        }
    }

    async fn handle(mut self, socket: Async<TcpStream>) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        S: Service<Request<Body>, Response = Response<Body>>
            + Clone 
            + Send 
            + Sync 
            + 'static,
        S::Future: Send + 'static,
        <S as Service<Request<Body>>>::Error: Send + Sync + std::error::Error,
    {
        let mut connection = server::handshake(socket).await?;
    
        while let Some(result) = connection.accept().await {
            let (mut request, mut respond) = result?;
    
            let body = request.body_mut();

            while let Some(data) = body.data().await {
                
                let data = data?;
                let datalen = data.len();

                let gd = Request::new(Body::new(Some(data)));
                let output = self.inner.call(gd).await?;

                let _ = body.flow_control().release_capacity(datalen);
                let output_bytes = output.into_inner().data();
                match output_bytes {
                    Some(data) => {
                        let hresponse = http::Response::new(());
                        let mut send = respond.send_response(hresponse, false)?;
                        send.send_data(data, false)?;
            
                        let mut trailers = HeaderMap::new();
                        trailers.insert("zomg", "hello".parse().unwrap());
            
                        send.send_trailers(trailers).unwrap();
                    }
                    None => {}
                }
            }
        }

        Ok(())
    }
}

use std::error::Error;
use std::net::TcpListener;
use cynthia::runtime::Async;

use lucat::transport::server::Server;
use lucat::common::{Request, Response, Status};

pub mod echo {
    lucat::include_proto!("echo");
}

use echo::{EchoRequest, EchoResponse};
use echo::echo_server::{Echo, EchoServer};

#[cynthia::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let listener = Async::<TcpListener>::bind("0.0.0.0:7000").await?;

    let mut server = Server::builder();
    let route = server.register(EchoServer::new(MyEcho::default()));
    println!("server listen on 0.0.0.0:7000");

    route.serve(listener).await?;

    Ok(())
}

#[derive(Default, Clone)]
pub struct MyEcho {}

#[lucat::async_trait]
impl Echo for MyEcho {
    async fn say_echo(
        &self, request: Request<EchoRequest>
    ) -> Result<Response<EchoResponse>, Status> {
        let r = request.into_inner();
        println!("recved: {:?}", r);
        let reply = EchoResponse {
            data: r.data,
            tag: r.tag,
            name: r.name,
        };

        Ok(Response::new(reply))
    }
}

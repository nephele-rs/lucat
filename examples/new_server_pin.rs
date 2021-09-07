use std::error::Error;
use std::net::TcpListener;
use cynthia::runtime::Async;
use pin_project_lite::pin_project;
use std::sync::{Arc, Mutex};
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
    let route = server.register(EchoServer::new(MyEcho::new()));
    println!("server listen on 0.0.0.0:7000");

    route.serve(listener).await?;

    Ok(())
}

#[derive(Default)]
struct GroupLock {
    pub count: Mutex<i32>,
}

pin_project! {
    #[derive(Default, Clone)]
    pub struct MyEcho {
        #[pin]
        inner: Arc<GroupLock>,
    }
}

impl MyEcho {
    fn new() -> Self {
        MyEcho {
            inner: Arc::new(GroupLock {
                count: Mutex::new(0),
            }),
        }
    }
}

#[lucat::async_trait]
impl Echo for MyEcho {
    async fn say_echo(
        &self, request: Request<EchoRequest>
    ) -> Result<Response<EchoResponse>, Status> {
        let mut count = self.inner.count.lock().unwrap();
        *count += 1;

        println!("count = {}", count);

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

use std::error::Error;
use std::net::{TcpListener, TcpStream};
use http::{HeaderMap};
use cynthia::runtime::{self, swap, Async};
use nephele::proto::h2::server;
use lucat::codec::prost;

pub mod echo {
    lucat::include_proto!("echo");
}

#[cynthia::main]
async fn main() -> swap::Result<()> {
    let listener = Async::<TcpListener>::bind("0.0.0.0:7000").await?;
    println!("h2 server listen on 0.0.0.0:7000");
    loop {
        let (stream, _peer_addr) = listener.accept().await?;

        runtime::spawn(handle(stream)).detach();
    }
}

async fn handle(socket: Async<TcpStream>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut connection = server::handshake(socket).await?;

    while let Some(result) = connection.accept().await {
        let (mut request, mut respond) = result?;

        let body = request.body_mut();
        while let Some(data) = body.data().await {
            
            let data = data?;
            let datalen = data.len();
            println!("data len = {} # {:?}", datalen, data);

            let gd = prost::decode(data).unwrap();

            println!("<<<< recv {:?}", gd);
            let _ = body.flow_control().release_capacity(datalen);

            let greeter = MyGreeter::default();
            let response = greeter.echo(gd).await?;

            let input_bytes = prost::encode(response).unwrap();
    
            let hresponse = http::Response::new(());
            let mut send = respond.send_response(hresponse, false)?;
            send.send_data(input_bytes, false)?;

            let mut trailers = HeaderMap::new();
            trailers.insert("zomg", "hello".parse().unwrap());

            send.send_trailers(trailers).unwrap();
        }
    }

    Ok(())
}

#[derive(Default)]
pub struct MyGreeter {}

impl MyGreeter {
    async fn echo(&self, request: echo::EchoRequest) -> Result<echo::EchoResponse, Box<dyn Error + Send + Sync>> {
        let reply = echo::EchoResponse {
            data: request.data,
            tag: request.tag,
            name: request.name,
        };

        Ok(reply)
    }
}

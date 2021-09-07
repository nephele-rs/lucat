use anyhow::Result;
use std::error::Error;
use lucat::common::{Request};

pub mod echo {
    lucat::include_proto!("echo");
}

use echo::{EchoRequest};
use echo::echo_client::{EchoClient};

#[cynthia::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut endpoint = EchoClient::connect("127.0.0.1:7000".to_string()).await?;

    let request = EchoRequest {
        data: vec![1, 2, 5],
        tag: vec![1],
        name: Some(150),
    };

    let response = endpoint.say_echo(Request::new(request)).await?;

    println!("response = {:?}", response);

    Ok(())
}
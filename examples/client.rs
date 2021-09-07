use anyhow::Result;
use http::{HeaderMap, Request};

use cynthia::runtime::{self, transport};
use nephele::proto::h2::client;
use http::{
    header::{HeaderValue, CONTENT_TYPE},
};

pub mod echo {
    lucat::include_proto!("echo");
}

use lucat::codec::prost;

#[cynthia::main]
async fn main() -> Result<()> {
    let tcp = transport::TcpStream::connect("127.0.0.1:7000").await?;
    let (mut client, h2) = client::handshake(tcp).await?;


    let grequest = echo::EchoRequest {
        data: vec![1, 2, 3],
        tag: vec![1],
        name: Some(150),
    };

    let input_bytes = prost::encode(grequest).unwrap();

    let mut request = Request::builder()
        .method("POST")
        .uri("http://127.0.0.1:7000")
        .body(())
        .unwrap();

    *request.version_mut() = http::Version::HTTP_2;
    
    request.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/grpc"));

    let mut trailers = HeaderMap::new();
    trailers.insert("zomg", "hello".parse().unwrap());

    let (response, mut stream) = client.send_request(request, false)?;
    stream.send_data(input_bytes, false)?;
    stream.send_trailers(trailers).unwrap();

    runtime::spawn(async move {
        if let Err(e) = h2.await {
            println!("GOT ERR={:?}", e);
        }
    })
    .detach();

    let response = response.await?;
    let mut body = response.into_body();

    while let Some(chunk) = body.data().await {

        let data = chunk?;
        let datalen = data.len();
        println!("data len = {} # {:?}", datalen, data);

        let gr = prost::decode::<echo::EchoResponse>(data);

        println!("GOT response = {:?}", gr.unwrap());

        let _ = body.flow_control().release_capacity(datalen);
    }

    if let Some(trailers) = body.trailers().await? {
        println!("GOT TRAILERS: {:?}", trailers);
    }

    Ok(())
}
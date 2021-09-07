use anyhow::Result;
use http::{HeaderMap, Request};
use cynthia::runtime::{self, transport};
use nephele::proto::h2::client;
use http::{
    header::{HeaderValue, CONTENT_TYPE},
};
use std::error::Error;
use crate::common::{self};
use crate::common::{Body, Response};
//use nephele::proto::h2::client::{SendRequest, Connection};
//use cynthia::runtime::TcpStream;

pub struct Endpoint {
    dst: String,
    //client: SendRequest<Bytes>,
    //connection: Connection<TcpStream>,
    //connection: Arc<RefCell<Connection<TcpStream>>>,
}


impl Endpoint {
    pub async fn connect(dst: String) -> Result<Self, crate::Error> {
        //let stream = transport::TcpStream::connect("127.0.0.1:7000").await?;
        //let (client, h2) = client::handshake(stream).await?;

        Ok(
            Endpoint {
                dst: dst.clone(),
                //client: client,
                //connection: h2,
                //connection: Arc::new(RefCell::new(h2)),
            }
        )
    }
}

#[crate::async_trait]
impl crate::SimpleInstantService for Endpoint {
    async fn call(&mut self, request: common::Request<Body>) -> Result<common::Response<Body>, crate::Error> {
        let res = self.request(request).await;
        res
    }
}

impl Endpoint {
    pub async fn request(&mut self, request: common::Request<Body>) -> Result<Response<Body>, Box<dyn Error + Send + Sync>> {
        let dst = self.dst.clone();
        let stream = transport::TcpStream::connect(dst).await?;
        let (mut h2client, h2) = client::handshake(stream).await?;
        
        let mut http_request = Request::builder()
            .method("POST")
            .uri("http://127.0.0.1:7000")
            .body(())
            .unwrap();

        *http_request.version_mut() = http::Version::HTTP_2;

        http_request.headers_mut()
            .insert(CONTENT_TYPE, HeaderValue::from_static("application/grpc"));

        let mut trailers = HeaderMap::new();
        trailers.insert("zomg", "hello".parse().unwrap());

        let (response, mut stream) = h2client.send_request(http_request, false)?;
        let input_bytes = request.into_inner().data();

        let input_bytes = match input_bytes {
            Some(b) => b,
            None => {
                return Ok(common::Response::new(Body::new(None)));
            }
        };


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
            
            let _ = body.flow_control().release_capacity(datalen);

            if let Some(_trailers) = body.trailers().await? {
            }

            let r = common::Response::new(Body::new(Some(data)));

            return Ok(r);
        }
    
        if let Some(_trailers) = body.trailers().await? {
        }
    
        let r = common::Response::new(Body::new(None));

        Ok(r)
    }
}

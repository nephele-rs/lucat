use http::{
    uri::{PathAndQuery},
};

use crate::codec::{Codec, Decoder, Encoder};
use crate::common::{Body, Request, Response};
use crate::{Status, Code};
use prost;

pub struct Rpc<T> {
    inner: T,
}

fn map_request<B, E>(
    encoder: &mut E,
    request: Request<B>,
) -> Result<Request<Body>, Status>
where
    B: prost::Message + Send + Sync + 'static,
    E: Encoder<Item = B, Error = Status> + Send + Sync + 'static,
{
    let res = request.into_inner();
    let output_bytes = encoder.encode(res).unwrap();

    let gd = Request::new(Body::new(Some(output_bytes)));
    Ok(gd)
}

impl<T> Rpc<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
        }
    }

    pub async fn unary<M1, M2, C>(
        &mut self, 
        req: Request<M1>,
        _path: PathAndQuery,
        mut codec: C,
    ) -> Result<Response<M2>, Status>
    where
        T: crate::SimpleInstantService + Send + Sync + 'static,
        C: Codec<Encode = M1, Decode = M2>,
        M1: Send + Sync + 'static + prost::Message,
        M2: Send + Sync + 'static,
    {
        let req = map_request(&mut codec.encoder(), req);

        match req {
            Ok(request) => {
                let response = self.inner.call(request).await;
                match response {
                    Ok(res) => {
                        let payload = res.into_inner().data();
                        match payload {
                            Some(data) => {
                                let mut decoder = codec.decoder();
                                let decoded_response = decoder.decode(data);
                                match decoded_response {
                                    Ok(Some(msg)) => {
                                        Ok(Response::new(msg))
                                    }
                                    Ok(None) => {
                                        Err(Status::new(Code::OutOfRange, "error"))
                                    }
                                    Err(_e) => {
                                        Err(Status::new(Code::OutOfRange, "error"))
                                    }
                                }
                            }
                            None => {
                                Err(Status::new(Code::OutOfRange, "error"))
                            }
                        }
                    }
                    Err(_) => {
                        Err(Status::new(Code::OutOfRange, "error"))
                    }
                }
            }
            Err(_) => {
                Err(Status::new(Code::OutOfRange, "error"))
            }
        }
    }
}

use crate::common::{Body, Request, Response};
use crate::controller::server::UnaryService;
use crate::codec::{Codec, Decoder, Encoder};
use crate::Status;
use prost;

fn map_response<B, E>(
    encoder: &mut E,
    response: Result<Response<B>, Status>,
) -> Result<Response<Body>, Status>
where
    B: prost::Message + Send + Sync + 'static,
    E: Encoder<Item = B, Error = Status> + Send + Sync + 'static,
{
    let response = match response {
        Ok(r) => r,
        Err(_) => return Ok(Response::new(Body::new(None))),
    };

    let res = response.into_inner();
    let output_bytes = encoder.encode(res).unwrap();

    let gd = Response::new(Body::new(Some(output_bytes)));
    Ok(gd)
}

pub struct Rpc<T> {
    codec: T,
}

impl<T> Rpc<T> 
where
    T: Codec,
    T::Encode: Sync,
{
    pub fn new(codec: T) -> Self {
        Self {
            codec,
        }
    }

    pub async fn unary<S>(
        &mut self,
        mut service: S,
        req: Request<Body>
    ) -> Response<Body> 
    where
        S: UnaryService<T::Decode, Response = T::Encode>,
        <T as Codec>::Encode: prost::Message,
    {
        let req = req.into_inner().data();
        match req {
            Some(data) => {
                let mut decoder = self.codec.decoder();
                let decoded_request = decoder.decode(data);

                match decoded_request {
                    Ok(Some(msg)) => {
                        let dec_req = Request::new(msg);
                        let output = service.call(dec_req).await;

                        let body = map_response(&mut self.codec.encoder(), output);
                        let gd = body.unwrap();
                        gd
                    }
                    Ok(None) => {
                        Response::new(Body::new(None))
                    }
                    Err(_e) => {
                        Response::new(Body::new(None))
                    }
                }
            }
            None => {
                Response::new(Body::new(None))
            }
        }
    }
}
use nephele::proto::h2::{RecvStream};
use bytes::{Bytes};

pub struct Body {
    pub data: Option<Bytes>,
}

pub enum BodyType {
    Once(Option<Bytes>),
    H2 {
        recv: RecvStream,
    },
}

impl Body {
    /*
    pub fn empty() -> Body {
        Body::new(BodyType::Once(None))
    }
    */

    pub fn new(data: Option<Bytes>) -> Body {
        Body { data }
    }

    /*
    pub fn h2(recv: h2::RecvStream) -> Self {
        let body = Body::new(BodyType::H2 {
            recv,
        });

        body
    }
    */

    pub fn data(self) -> Option<Bytes> {
        self.data
    }

    /*
    pub async fn get_data(mut self) -> Option<Result<Bytes, h2::Error>> {
        self.recv.data().await
    }
    */
}

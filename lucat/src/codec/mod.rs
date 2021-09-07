pub mod buffer;
pub mod prost;

pub use self::buffer::{DecodeBuf, EncodeBuf};
pub use crate::common::{self, Body, Request, Response};

pub mod error;

pub use crate::codec::prost::ProstCodec;
use crate::Status;

use std::io;

pub trait Codec: Default {
    type Encode: Send + 'static;
    type Decode: Send + 'static;

    type Encoder: Encoder<Item = Self::Encode, Error = Status> + Send + Sync + 'static;
    type Decoder: Decoder<Item = Self::Decode, Error = Status> + Send + Sync + 'static;

    fn encoder(&mut self) -> Self::Encoder;
    fn decoder(&mut self) -> Self::Decoder;
}

pub trait Encoder {
    type Item;

    type Error: From<io::Error>;

    fn encode(&mut self, item: Self::Item) -> Result<bytes::Bytes, Self::Error>;
}

pub trait Decoder {
    type Item;

    type Error: From<io::Error>;

    fn decode(&mut self, src: bytes::Bytes) -> Result<Option<Self::Item>, Self::Error>;
}

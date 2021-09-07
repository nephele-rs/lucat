use std::marker::PhantomData;
use prost::Message;
use super::{Codec, Decoder, Encoder};
use crate::{Code, Status};
use super::error;

#[derive(Debug, Clone)]
pub struct ProstCodec<T, U> {
    _pd: PhantomData<(T, U)>,
}

impl<T, U> Default for ProstCodec<T, U> {
    fn default() -> Self {
        Self { _pd: PhantomData }
    }
}

impl<T, U> Codec for ProstCodec<T, U>
where
    T: Message + Send + 'static,
    U: Message + Default + Send + 'static,
{
    type Encode = T;
    type Decode = U;

    type Encoder = ProstEncoder<T>;
    type Decoder = ProstDecoder<U>;

    fn encoder(&mut self) -> Self::Encoder {
        ProstEncoder(PhantomData)
    }

    fn decoder(&mut self) -> Self::Decoder {
        ProstDecoder(PhantomData)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProstEncoder<T>(PhantomData<T>);

impl<T: Message> Encoder for ProstEncoder<T> {
    type Item = T;
    type Error = Status;

    fn encode(&mut self, item: Self::Item) -> Result<bytes::Bytes, Self::Error> {
        let len = Message::encoded_len(&item);
        let mut buf = ::bytes::BytesMut::with_capacity(len);
        item.encode(&mut buf).expect("Message only errors if not enough space");
        Ok(buf.freeze())
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProstDecoder<U>(PhantomData<U>);

impl<U: Message + Default> Decoder for ProstDecoder<U> {
    type Item = U;
    type Error = Status;

    fn decode(&mut self, buf: bytes::Bytes) -> Result<Option<Self::Item>, Self::Error> {
        let item = Message::decode(buf)
                    .map(Option::Some)
                    .map_err(from_decode_error)?;

        Ok(item)
    }
}

fn from_decode_error(error: prost::DecodeError) -> crate::Status {
    Status::new(Code::Internal, error.to_string())
}

pub fn decode<M>(buf: bytes::Bytes) -> Result<M, prost::DecodeError>
where
    M: prost::Message + Default,
{
    let message = prost::Message::decode(buf)?;
    Ok(message)
}

pub fn encode<M>(message: M) -> error::Result<bytes::Bytes, prost::DecodeError>
where
    M: prost::Message,
{
    let len = prost::Message::encoded_len(&message);
    let mut buf = ::bytes::BytesMut::with_capacity(len);
    prost::Message::encode(&message, &mut buf)?;
    Ok(buf.freeze())
}

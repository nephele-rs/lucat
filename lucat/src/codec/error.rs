use std::result;

use failure;
use prost;

pub type Result<A, E> = result::Result<A, Error<E>>;

#[derive(Clone, Debug, Eq, failure::Fail, PartialEq)]
pub enum Error<E>
where
    E: failure::Fail,
{
    #[fail(display = "Execution error: {}", error)]
    Execution {
        #[cause]
        error: E,
    },
    #[fail(display = "Decode error: {}", error)]
    Decode {
        #[cause]
        error: prost::DecodeError,
    },
    #[fail(display = "Encode error: {}", error)]
    Encode {
        #[cause]
        error: prost::EncodeError,
    },
}

impl<E> Error<E>
where
    E: failure::Fail,
{
    pub fn execution(error: E) -> Self {
        Error::Execution { error }
    }
}

impl<E> From<prost::DecodeError> for Error<E>
where
    E: failure::Fail,
{
    fn from(error: prost::DecodeError) -> Self {
        Error::Decode { error }
    }
}

impl<E> From<prost::EncodeError> for Error<E>
where
    E: failure::Fail,
{
    fn from(error: prost::EncodeError) -> Self {
        Error::Encode { error }
    }
}

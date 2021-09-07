use bytes::Bytes;
use http::header::HeaderName;
use std::borrow::Borrow;
use std::error::Error;
use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

use super::encoding::{Ascii, Binary, ValueEncoding};

#[derive(Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct MetadataKey<VE: ValueEncoding> {
    pub(crate) inner: http::header::HeaderName,
    phantom: PhantomData<VE>,
}

#[derive(Debug)]
pub struct InvalidMetadataKey {
    _priv: (),
}

pub type AsciiMetadataKey = MetadataKey<Ascii>;
pub type BinaryMetadataKey = MetadataKey<Binary>;

impl<VE: ValueEncoding> MetadataKey<VE> {
    pub fn from_bytes(src: &[u8]) -> Result<Self, InvalidMetadataKey> {
        match HeaderName::from_bytes(src) {
            Ok(name) => {
                if !VE::is_valid_key(name.as_str()) {
                    panic!("invalid metadata key")
                }

                Ok(MetadataKey {
                    inner: name,
                    phantom: PhantomData,
                })
            }
            Err(_) => Err(InvalidMetadataKey::new()),
        }
    }

    pub fn from_static(src: &'static str) -> Self {
        let name = HeaderName::from_static(src);
        if !VE::is_valid_key(name.as_str()) {
            panic!("invalid metadata key")
        }

        MetadataKey {
            inner: name,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }

    #[inline]
    pub(crate) fn unchecked_from_header_name_ref(header_name: &HeaderName) -> &Self {
        unsafe { &*(header_name as *const HeaderName as *const Self) }
    }

    #[inline]
    pub(crate) fn unchecked_from_header_name(name: HeaderName) -> Self {
        MetadataKey {
            inner: name,
            phantom: PhantomData,
        }
    }
}

impl<VE: ValueEncoding> FromStr for MetadataKey<VE> {
    type Err = InvalidMetadataKey;

    fn from_str(s: &str) -> Result<Self, InvalidMetadataKey> {
        MetadataKey::from_bytes(s.as_bytes()).map_err(|_| InvalidMetadataKey::new())
    }
}

impl<VE: ValueEncoding> AsRef<str> for MetadataKey<VE> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<VE: ValueEncoding> AsRef<[u8]> for MetadataKey<VE> {
    fn as_ref(&self) -> &[u8] {
        self.as_str().as_bytes()
    }
}

impl<VE: ValueEncoding> Borrow<str> for MetadataKey<VE> {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<VE: ValueEncoding> fmt::Debug for MetadataKey<VE> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), fmt)
    }
}

impl<VE: ValueEncoding> fmt::Display for MetadataKey<VE> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), fmt)
    }
}

impl InvalidMetadataKey {
    #[doc(hidden)]
    pub fn new() -> InvalidMetadataKey {
        InvalidMetadataKey { _priv: () }
    }
}

impl<'a, VE: ValueEncoding> From<&'a MetadataKey<VE>> for MetadataKey<VE> {
    fn from(src: &'a MetadataKey<VE>) -> MetadataKey<VE> {
        src.clone()
    }
}

impl<VE: ValueEncoding> From<MetadataKey<VE>> for Bytes {
    #[inline]
    fn from(name: MetadataKey<VE>) -> Bytes {
        Bytes::copy_from_slice(name.inner.as_ref())
    }
}

impl<'a, VE: ValueEncoding> PartialEq<&'a MetadataKey<VE>> for MetadataKey<VE> {
    #[inline]
    fn eq(&self, other: &&'a MetadataKey<VE>) -> bool {
        *self == **other
    }
}

impl<'a, VE: ValueEncoding> PartialEq<MetadataKey<VE>> for &'a MetadataKey<VE> {
    #[inline]
    fn eq(&self, other: &MetadataKey<VE>) -> bool {
        *other == *self
    }
}

impl<VE: ValueEncoding> PartialEq<str> for MetadataKey<VE> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.inner.eq(other)
    }
}

impl<VE: ValueEncoding> PartialEq<MetadataKey<VE>> for str {
    #[inline]
    fn eq(&self, other: &MetadataKey<VE>) -> bool {
        (*other).inner == *self
    }
}

impl<'a, VE: ValueEncoding> PartialEq<&'a str> for MetadataKey<VE> {
    #[inline]
    fn eq(&self, other: &&'a str) -> bool {
        *self == **other
    }
}

impl<'a, VE: ValueEncoding> PartialEq<MetadataKey<VE>> for &'a str {
    #[inline]
    fn eq(&self, other: &MetadataKey<VE>) -> bool {
        *other == *self
    }
}

impl fmt::Display for InvalidMetadataKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid gRPC metadata key name")
    }
}

impl Default for InvalidMetadataKey {
    fn default() -> Self {
        Self::new()
    }
}

impl Error for InvalidMetadataKey {}

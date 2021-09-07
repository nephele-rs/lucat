#![allow(clippy::upper_case_acronyms)]

use super::encoding::{
    Ascii, Binary, InvalidMetadataValue, InvalidMetadataValueBytes, ValueEncoding,
};
use super::key::MetadataKey;

use bytes::Bytes;
use http::header::HeaderValue;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::str::FromStr;
use std::{cmp, fmt};

#[derive(Clone)]
#[repr(transparent)]
pub struct MetadataValue<VE: ValueEncoding> {
    pub(crate) inner: HeaderValue,
    phantom: PhantomData<VE>,
}

#[derive(Debug)]
pub struct ToStrError {
    _priv: (),
}

pub type AsciiMetadataValue = MetadataValue<Ascii>;
pub type BinaryMetadataValue = MetadataValue<Binary>;

impl<VE: ValueEncoding> MetadataValue<VE> {
    #[inline]
    pub fn from_static(src: &'static str) -> Self {
        MetadataValue {
            inner: VE::from_static(src),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn try_from_bytes(src: &[u8]) -> Result<Self, InvalidMetadataValueBytes> {
        VE::from_bytes(src).map(|value| MetadataValue {
            inner: value,
            phantom: PhantomData,
        })
    }

    #[inline]
    pub fn from_shared(src: Bytes) -> Result<Self, InvalidMetadataValueBytes> {
        VE::from_shared(src).map(|value| MetadataValue {
            inner: value,
            phantom: PhantomData,
        })
    }

    #[inline]
    pub unsafe fn from_shared_unchecked(src: Bytes) -> Self {
        MetadataValue {
            inner: HeaderValue::from_maybe_shared_unchecked(src),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        VE::is_empty(self.inner.as_bytes())
    }

    #[inline]
    pub fn to_bytes(&self) -> Result<Bytes, InvalidMetadataValueBytes> {
        VE::decode(self.inner.as_bytes())
    }

    #[inline]
    pub fn set_sensitive(&mut self, val: bool) {
        self.inner.set_sensitive(val);
    }

    #[inline]
    pub fn is_sensitive(&self) -> bool {
        self.inner.is_sensitive()
    }

    #[inline]
    pub fn as_encoded_bytes(&self) -> &[u8] {
        self.inner.as_bytes()
    }

    #[inline]
    pub(crate) fn unchecked_from_header_value(value: HeaderValue) -> Self {
        MetadataValue {
            inner: value,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub(crate) fn unchecked_from_header_value_ref(header_value: &HeaderValue) -> &Self {
        unsafe { &*(header_value as *const HeaderValue as *const Self) }
    }

    #[inline]
    pub(crate) fn unchecked_from_mut_header_value_ref(header_value: &mut HeaderValue) -> &mut Self {
        unsafe { &mut *(header_value as *mut HeaderValue as *mut Self) }
    }
}

#[allow(clippy::len_without_is_empty)]
impl MetadataValue<Ascii> {
    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn from_str(src: &str) -> Result<Self, InvalidMetadataValue> {
        HeaderValue::from_str(src)
            .map(|value| MetadataValue {
                inner: value,
                phantom: PhantomData,
            })
            .map_err(|_| InvalidMetadataValue::new())
    }

    #[inline]
    pub fn from_key<KeyVE: ValueEncoding>(key: MetadataKey<KeyVE>) -> Self {
        key.into()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn to_str(&self) -> Result<&str, ToStrError> {
        self.inner.to_str().map_err(|_| ToStrError::new())
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.inner.as_bytes()
    }
}

impl MetadataValue<Binary> {
    #[inline]
    pub fn from_bytes(src: &[u8]) -> Self {
        Self::try_from_bytes(src).unwrap()
    }
}

impl<VE: ValueEncoding> AsRef<[u8]> for MetadataValue<VE> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.inner.as_ref()
    }
}

impl<VE: ValueEncoding> fmt::Debug for MetadataValue<VE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        VE::fmt(&self.inner, f)
    }
}

impl<KeyVE: ValueEncoding> From<MetadataKey<KeyVE>> for MetadataValue<Ascii> {
    #[inline]
    fn from(h: MetadataKey<KeyVE>) -> MetadataValue<Ascii> {
        MetadataValue {
            inner: h.inner.into(),
            phantom: PhantomData,
        }
    }
}

macro_rules! from_integers {
    ($($name:ident: $t:ident => $max_len:expr),*) => {$(
        impl From<$t> for MetadataValue<Ascii> {
            fn from(num: $t) -> MetadataValue<Ascii> {
                MetadataValue {
                    inner: HeaderValue::from(num),
                    phantom: PhantomData,
                }
            }
        }

        #[test]
        fn $name() {
            let n: $t = 55;
            let val = AsciiMetadataValue::from(n);
            assert_eq!(val, &n.to_string());

            let n = ::std::$t::MAX;
            let val = AsciiMetadataValue::from(n);
            assert_eq!(val, &n.to_string());
        }
    )*};
}

from_integers! {
    from_u16: u16 => 5,
    from_i16: i16 => 6,
    from_u32: u32 => 10,
    from_i32: i32 => 11,
    from_u64: u64 => 20,
    from_i64: i64 => 20
}

#[cfg(target_pointer_width = "16")]
from_integers! {
    from_usize: usize => 5,
    from_isize: isize => 6
}

#[cfg(target_pointer_width = "32")]
from_integers! {
    from_usize: usize => 10,
    from_isize: isize => 11
}

#[cfg(target_pointer_width = "64")]
from_integers! {
    from_usize: usize => 20,
    from_isize: isize => 20
}

#[cfg(test)]
mod from_metadata_value_tests {
    use super::*;
    use crate::metadata::map::MetadataMap;

    #[test]
    fn it_can_insert_metadata_key_as_metadata_value() {
        let mut map = MetadataMap::new();
        map.insert(
            "accept",
            MetadataKey::<Ascii>::from_bytes(b"hello-world")
                .unwrap()
                .into(),
        );

        assert_eq!(
            map.get("accept").unwrap(),
            AsciiMetadataValue::try_from_bytes(b"hello-world").unwrap()
        );
    }
}

impl FromStr for MetadataValue<Ascii> {
    type Err = InvalidMetadataValue;

    #[inline]
    fn from_str(s: &str) -> Result<MetadataValue<Ascii>, Self::Err> {
        MetadataValue::<Ascii>::from_str(s)
    }
}

impl<VE: ValueEncoding> From<MetadataValue<VE>> for Bytes {
    #[inline]
    fn from(value: MetadataValue<VE>) -> Bytes {
        Bytes::copy_from_slice(value.inner.as_bytes())
    }
}

impl<'a, VE: ValueEncoding> From<&'a MetadataValue<VE>> for MetadataValue<VE> {
    #[inline]
    fn from(t: &'a MetadataValue<VE>) -> Self {
        t.clone()
    }
}

impl ToStrError {
    pub(crate) fn new() -> Self {
        ToStrError { _priv: () }
    }
}

impl fmt::Display for ToStrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("failed to convert metadata to a str")
    }
}

impl Error for ToStrError {}

impl Hash for MetadataValue<Ascii> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state)
    }
}

impl Hash for MetadataValue<Binary> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self.to_bytes() {
            Ok(b) => b.hash(state),
            Err(e) => e.hash(state),
        }
    }
}

impl<VE: ValueEncoding> PartialEq for MetadataValue<VE> {
    #[inline]
    fn eq(&self, other: &MetadataValue<VE>) -> bool {
        VE::values_equal(&self.inner, &other.inner)
    }
}

impl<VE: ValueEncoding> Eq for MetadataValue<VE> {}

impl<VE: ValueEncoding> PartialOrd for MetadataValue<VE> {
    #[inline]
    fn partial_cmp(&self, other: &MetadataValue<VE>) -> Option<cmp::Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

impl<VE: ValueEncoding> Ord for MetadataValue<VE> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl<VE: ValueEncoding> PartialEq<str> for MetadataValue<VE> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        VE::equals(&self.inner, other.as_bytes())
    }
}

impl<VE: ValueEncoding> PartialEq<[u8]> for MetadataValue<VE> {
    #[inline]
    fn eq(&self, other: &[u8]) -> bool {
        VE::equals(&self.inner, other)
    }
}

impl<VE: ValueEncoding> PartialOrd<str> for MetadataValue<VE> {
    #[inline]
    fn partial_cmp(&self, other: &str) -> Option<cmp::Ordering> {
        self.inner.partial_cmp(other.as_bytes())
    }
}

impl<VE: ValueEncoding> PartialOrd<[u8]> for MetadataValue<VE> {
    #[inline]
    fn partial_cmp(&self, other: &[u8]) -> Option<cmp::Ordering> {
        self.inner.partial_cmp(other)
    }
}

impl<VE: ValueEncoding> PartialEq<MetadataValue<VE>> for str {
    #[inline]
    fn eq(&self, other: &MetadataValue<VE>) -> bool {
        *other == *self
    }
}

impl<VE: ValueEncoding> PartialEq<MetadataValue<VE>> for [u8] {
    #[inline]
    fn eq(&self, other: &MetadataValue<VE>) -> bool {
        *other == *self
    }
}

impl<VE: ValueEncoding> PartialOrd<MetadataValue<VE>> for str {
    #[inline]
    fn partial_cmp(&self, other: &MetadataValue<VE>) -> Option<cmp::Ordering> {
        self.as_bytes().partial_cmp(other.inner.as_bytes())
    }
}

impl<VE: ValueEncoding> PartialOrd<MetadataValue<VE>> for [u8] {
    #[inline]
    fn partial_cmp(&self, other: &MetadataValue<VE>) -> Option<cmp::Ordering> {
        self.partial_cmp(other.inner.as_bytes())
    }
}

impl<VE: ValueEncoding> PartialEq<String> for MetadataValue<VE> {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        *self == other[..]
    }
}

impl<VE: ValueEncoding> PartialOrd<String> for MetadataValue<VE> {
    #[inline]
    fn partial_cmp(&self, other: &String) -> Option<cmp::Ordering> {
        self.inner.partial_cmp(other.as_bytes())
    }
}

impl<VE: ValueEncoding> PartialEq<MetadataValue<VE>> for String {
    #[inline]
    fn eq(&self, other: &MetadataValue<VE>) -> bool {
        *other == *self
    }
}

impl<VE: ValueEncoding> PartialOrd<MetadataValue<VE>> for String {
    #[inline]
    fn partial_cmp(&self, other: &MetadataValue<VE>) -> Option<cmp::Ordering> {
        self.as_bytes().partial_cmp(other.inner.as_bytes())
    }
}

impl<'a, VE: ValueEncoding> PartialEq<MetadataValue<VE>> for &'a MetadataValue<VE> {
    #[inline]
    fn eq(&self, other: &MetadataValue<VE>) -> bool {
        **self == *other
    }
}

impl<'a, VE: ValueEncoding> PartialOrd<MetadataValue<VE>> for &'a MetadataValue<VE> {
    #[inline]
    fn partial_cmp(&self, other: &MetadataValue<VE>) -> Option<cmp::Ordering> {
        (**self).partial_cmp(other)
    }
}

impl<'a, VE: ValueEncoding, T: ?Sized> PartialEq<&'a T> for MetadataValue<VE>
where
    MetadataValue<VE>: PartialEq<T>,
{
    #[inline]
    fn eq(&self, other: &&'a T) -> bool {
        *self == **other
    }
}

impl<'a, VE: ValueEncoding, T: ?Sized> PartialOrd<&'a T> for MetadataValue<VE>
where
    MetadataValue<VE>: PartialOrd<T>,
{
    #[inline]
    fn partial_cmp(&self, other: &&'a T) -> Option<cmp::Ordering> {
        self.partial_cmp(*other)
    }
}

impl<'a, VE: ValueEncoding> PartialEq<MetadataValue<VE>> for &'a str {
    #[inline]
    fn eq(&self, other: &MetadataValue<VE>) -> bool {
        *other == *self
    }
}

impl<'a, VE: ValueEncoding> PartialOrd<MetadataValue<VE>> for &'a str {
    #[inline]
    fn partial_cmp(&self, other: &MetadataValue<VE>) -> Option<cmp::Ordering> {
        self.as_bytes().partial_cmp(other.inner.as_bytes())
    }
}

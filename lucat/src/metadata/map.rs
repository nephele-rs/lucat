pub(crate) use self::as_encoding_agnostic_metadata_key::AsEncodingAgnosticMetadataKey;
pub(crate) use self::as_metadata_key::AsMetadataKey;
pub(crate) use self::into_metadata_key::IntoMetadataKey;

use super::encoding::{Ascii, Binary, ValueEncoding};
use super::key::{InvalidMetadataKey, MetadataKey};
use super::value::MetadataValue;

use std::marker::PhantomData;

#[derive(Clone, Debug, Default)]
pub struct MetadataMap {
    headers: http::HeaderMap,
}

#[derive(Debug)]
pub struct Iter<'a> {
    inner: http::header::Iter<'a, http::header::HeaderValue>,
}

#[derive(Debug)]
pub enum KeyAndValueRef<'a> {
    Ascii(&'a MetadataKey<Ascii>, &'a MetadataValue<Ascii>),
    Binary(&'a MetadataKey<Binary>, &'a MetadataValue<Binary>),
}

#[derive(Debug)]
pub enum KeyAndMutValueRef<'a> {
    Ascii(&'a MetadataKey<Ascii>, &'a mut MetadataValue<Ascii>),
    Binary(&'a MetadataKey<Binary>, &'a mut MetadataValue<Binary>),
}

#[derive(Debug)]
pub struct IterMut<'a> {
    inner: http::header::IterMut<'a, http::header::HeaderValue>,
}

#[derive(Debug)]
pub struct ValueDrain<'a, VE: ValueEncoding> {
    inner: http::header::ValueDrain<'a, http::header::HeaderValue>,
    phantom: PhantomData<VE>,
}

#[derive(Debug)]
pub struct Keys<'a> {
    inner: http::header::Keys<'a, http::header::HeaderValue>,
}

#[derive(Debug)]
pub enum KeyRef<'a> {
    Ascii(&'a MetadataKey<Ascii>),
    Binary(&'a MetadataKey<Binary>),
}

#[derive(Debug)]
pub struct Values<'a> {
    inner: http::header::Iter<'a, http::header::HeaderValue>,
}

#[derive(Debug)]
pub enum ValueRef<'a> {
    Ascii(&'a MetadataValue<Ascii>),
    Binary(&'a MetadataValue<Binary>),
}

#[derive(Debug)]
pub struct ValuesMut<'a> {
    inner: http::header::IterMut<'a, http::header::HeaderValue>,
}

#[derive(Debug)]
pub enum ValueRefMut<'a> {
    Ascii(&'a mut MetadataValue<Ascii>),
    Binary(&'a mut MetadataValue<Binary>),
}

#[derive(Debug)]
pub struct ValueIter<'a, VE: ValueEncoding> {
    inner: Option<http::header::ValueIter<'a, http::header::HeaderValue>>,
    phantom: PhantomData<VE>,
}

#[derive(Debug)]
pub struct ValueIterMut<'a, VE: ValueEncoding> {
    inner: http::header::ValueIterMut<'a, http::header::HeaderValue>,
    phantom: PhantomData<VE>,
}

#[derive(Debug)]
pub struct GetAll<'a, VE: ValueEncoding> {
    inner: Option<http::header::GetAll<'a, http::header::HeaderValue>>,
    phantom: PhantomData<VE>,
}

#[derive(Debug)]
pub enum Entry<'a, VE: ValueEncoding> {
    Occupied(OccupiedEntry<'a, VE>),
    Vacant(VacantEntry<'a, VE>),
}

#[derive(Debug)]
pub struct VacantEntry<'a, VE: ValueEncoding> {
    inner: http::header::VacantEntry<'a, http::header::HeaderValue>,
    phantom: PhantomData<VE>,
}

#[derive(Debug)]
pub struct OccupiedEntry<'a, VE: ValueEncoding> {
    inner: http::header::OccupiedEntry<'a, http::header::HeaderValue>,
    phantom: PhantomData<VE>,
}

pub const GRPC_TIMEOUT_HEADER: &str = "grpc-timeout";

impl MetadataMap {
    pub(crate) const GRPC_RESERVED_HEADERS: [&'static str; 6] = [
        "te",
        "user-agent",
        "content-type",
        "grpc-message",
        "grpc-message-type",
        "grpc-status",
    ];

    pub fn new() -> Self {
        MetadataMap::with_capacity(0)
    }

    pub fn from_headers(headers: http::HeaderMap) -> Self {
        MetadataMap { headers }
    }

    pub fn into_headers(self) -> http::HeaderMap {
        self.headers
    }

    pub(crate) fn into_sanitized_headers(mut self) -> http::HeaderMap {
        for r in &Self::GRPC_RESERVED_HEADERS {
            self.headers.remove(*r);
        }
        self.headers
    }

    pub fn with_capacity(capacity: usize) -> MetadataMap {
        MetadataMap {
            headers: http::HeaderMap::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.headers.len()
    }

    pub fn keys_len(&self) -> usize {
        self.headers.keys_len()
    }

    pub fn is_empty(&self) -> bool {
        self.headers.is_empty()
    }

    pub fn clear(&mut self) {
        self.headers.clear();
    }

    pub fn capacity(&self) -> usize {
        self.headers.capacity()
    }

    pub fn reserve(&mut self, additional: usize) {
        self.headers.reserve(additional);
    }

    pub fn get<K>(&self, key: K) -> Option<&MetadataValue<Ascii>>
    where
        K: AsMetadataKey<Ascii>,
    {
        key.get(self)
    }

    pub fn get_bin<K>(&self, key: K) -> Option<&MetadataValue<Binary>>
    where
        K: AsMetadataKey<Binary>,
    {
        key.get(self)
    }

    pub fn get_mut<K>(&mut self, key: K) -> Option<&mut MetadataValue<Ascii>>
    where
        K: AsMetadataKey<Ascii>,
    {
        key.get_mut(self)
    }

    pub fn get_bin_mut<K>(&mut self, key: K) -> Option<&mut MetadataValue<Binary>>
    where
        K: AsMetadataKey<Binary>,
    {
        key.get_mut(self)
    }

    pub fn get_all<K>(&self, key: K) -> GetAll<'_, Ascii>
    where
        K: AsMetadataKey<Ascii>,
    {
        GetAll {
            inner: key.get_all(self),
            phantom: PhantomData,
        }
    }

    pub fn get_all_bin<K>(&self, key: K) -> GetAll<'_, Binary>
    where
        K: AsMetadataKey<Binary>,
    {
        GetAll {
            inner: key.get_all(self),
            phantom: PhantomData,
        }
    }

    pub fn contains_key<K>(&self, key: K) -> bool
    where
        K: AsEncodingAgnosticMetadataKey,
    {
        key.contains_key(self)
    }

    pub fn iter(&self) -> Iter<'_> {
        Iter {
            inner: self.headers.iter(),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_> {
        IterMut {
            inner: self.headers.iter_mut(),
        }
    }

    pub fn keys(&self) -> Keys<'_> {
        Keys {
            inner: self.headers.keys(),
        }
    }

    pub fn values(&self) -> Values<'_> {
        Values {
            inner: self.headers.iter(),
        }
    }

    pub fn values_mut(&mut self) -> ValuesMut<'_> {
        ValuesMut {
            inner: self.headers.iter_mut(),
        }
    }

    pub fn entry<K>(&mut self, key: K) -> Result<Entry<'_, Ascii>, InvalidMetadataKey>
    where
        K: AsMetadataKey<Ascii>,
    {
        self.generic_entry::<Ascii, K>(key)
    }

    pub fn entry_bin<K>(&mut self, key: K) -> Result<Entry<'_, Binary>, InvalidMetadataKey>
    where
        K: AsMetadataKey<Binary>,
    {
        self.generic_entry::<Binary, K>(key)
    }

    fn generic_entry<VE: ValueEncoding, K>(
        &mut self,
        key: K,
    ) -> Result<Entry<'_, VE>, InvalidMetadataKey>
    where
        K: AsMetadataKey<VE>,
    {
        match key.entry(self) {
            Ok(entry) => Ok(match entry {
                http::header::Entry::Occupied(e) => Entry::Occupied(OccupiedEntry {
                    inner: e,
                    phantom: PhantomData,
                }),
                http::header::Entry::Vacant(e) => Entry::Vacant(VacantEntry {
                    inner: e,
                    phantom: PhantomData,
                }),
            }),
            Err(err) => Err(err),
        }
    }

    pub fn insert<K>(&mut self, key: K, val: MetadataValue<Ascii>) -> Option<MetadataValue<Ascii>>
    where
        K: IntoMetadataKey<Ascii>,
    {
        key.insert(self, val)
    }

    pub fn insert_bin<K>(
        &mut self,
        key: K,
        val: MetadataValue<Binary>,
    ) -> Option<MetadataValue<Binary>>
    where
        K: IntoMetadataKey<Binary>,
    {
        key.insert(self, val)
    }

    pub fn append<K>(&mut self, key: K, value: MetadataValue<Ascii>) -> bool
    where
        K: IntoMetadataKey<Ascii>,
    {
        key.append(self, value)
    }

    pub fn append_bin<K>(&mut self, key: K, value: MetadataValue<Binary>) -> bool
    where
        K: IntoMetadataKey<Binary>,
    {
        key.append(self, value)
    }

    pub fn remove<K>(&mut self, key: K) -> Option<MetadataValue<Ascii>>
    where
        K: AsMetadataKey<Ascii>,
    {
        key.remove(self)
    }

    pub fn remove_bin<K>(&mut self, key: K) -> Option<MetadataValue<Binary>>
    where
        K: AsMetadataKey<Binary>,
    {
        key.remove(self)
    }

    pub fn merge(&mut self, other: MetadataMap) {
        self.headers.extend(other.headers);
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = KeyAndValueRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|item| {
            let (ref name, value) = item;
            if Ascii::is_valid_key(name.as_str()) {
                KeyAndValueRef::Ascii(
                    MetadataKey::unchecked_from_header_name_ref(name),
                    MetadataValue::unchecked_from_header_value_ref(value),
                )
            } else {
                KeyAndValueRef::Binary(
                    MetadataKey::unchecked_from_header_name_ref(name),
                    MetadataValue::unchecked_from_header_value_ref(value),
                )
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a> Iterator for IterMut<'a> {
    type Item = KeyAndMutValueRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|item| {
            let (name, value) = item;
            if Ascii::is_valid_key(name.as_str()) {
                KeyAndMutValueRef::Ascii(
                    MetadataKey::unchecked_from_header_name_ref(name),
                    MetadataValue::unchecked_from_mut_header_value_ref(value),
                )
            } else {
                KeyAndMutValueRef::Binary(
                    MetadataKey::unchecked_from_header_name_ref(name),
                    MetadataValue::unchecked_from_mut_header_value_ref(value),
                )
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, VE: ValueEncoding> Iterator for ValueDrain<'a, VE> {
    type Item = MetadataValue<VE>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(MetadataValue::unchecked_from_header_value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a> Iterator for Keys<'a> {
    type Item = KeyRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|key| {
            if Ascii::is_valid_key(key.as_str()) {
                KeyRef::Ascii(MetadataKey::unchecked_from_header_name_ref(key))
            } else {
                KeyRef::Binary(MetadataKey::unchecked_from_header_name_ref(key))
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a> ExactSizeIterator for Keys<'a> {}

impl<'a> Iterator for Values<'a> {
    type Item = ValueRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|item| {
            let (ref name, value) = item;
            if Ascii::is_valid_key(name.as_str()) {
                ValueRef::Ascii(MetadataValue::unchecked_from_header_value_ref(value))
            } else {
                ValueRef::Binary(MetadataValue::unchecked_from_header_value_ref(value))
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a> Iterator for ValuesMut<'a> {
    type Item = ValueRefMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|item| {
            let (name, value) = item;
            if Ascii::is_valid_key(name.as_str()) {
                ValueRefMut::Ascii(MetadataValue::unchecked_from_mut_header_value_ref(value))
            } else {
                ValueRefMut::Binary(MetadataValue::unchecked_from_mut_header_value_ref(value))
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, VE: ValueEncoding> Iterator for ValueIter<'a, VE>
where
    VE: 'a,
{
    type Item = &'a MetadataValue<VE>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner {
            Some(ref mut inner) => inner
                .next()
                .map(&MetadataValue::unchecked_from_header_value_ref),
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.inner {
            Some(ref inner) => inner.size_hint(),
            None => (0, Some(0)),
        }
    }
}

impl<'a, VE: ValueEncoding> DoubleEndedIterator for ValueIter<'a, VE>
where
    VE: 'a,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.inner {
            Some(ref mut inner) => inner
                .next_back()
                .map(&MetadataValue::unchecked_from_header_value_ref),
            None => None,
        }
    }
}

impl<'a, VE: ValueEncoding> Iterator for ValueIterMut<'a, VE>
where
    VE: 'a,
{
    type Item = &'a mut MetadataValue<VE>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(&MetadataValue::unchecked_from_mut_header_value_ref)
    }
}

impl<'a, VE: ValueEncoding> DoubleEndedIterator for ValueIterMut<'a, VE>
where
    VE: 'a,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner
            .next_back()
            .map(&MetadataValue::unchecked_from_mut_header_value_ref)
    }
}

impl<'a, VE: ValueEncoding> Entry<'a, VE> {
    pub fn or_insert(self, default: MetadataValue<VE>) -> &'a mut MetadataValue<VE> {
        use self::Entry::*;

        match self {
            Occupied(e) => e.into_mut(),
            Vacant(e) => e.insert(default),
        }
    }

    pub fn or_insert_with<F: FnOnce() -> MetadataValue<VE>>(
        self,
        default: F,
    ) -> &'a mut MetadataValue<VE> {
        use self::Entry::*;

        match self {
            Occupied(e) => e.into_mut(),
            Vacant(e) => e.insert(default()),
        }
    }

    pub fn key(&self) -> &MetadataKey<VE> {
        use self::Entry::*;

        MetadataKey::unchecked_from_header_name_ref(match *self {
            Vacant(ref e) => e.inner.key(),
            Occupied(ref e) => e.inner.key(),
        })
    }
}

impl<'a, VE: ValueEncoding> VacantEntry<'a, VE> {
    pub fn key(&self) -> &MetadataKey<VE> {
        MetadataKey::unchecked_from_header_name_ref(self.inner.key())
    }

    pub fn into_key(self) -> MetadataKey<VE> {
        MetadataKey::unchecked_from_header_name(self.inner.into_key())
    }

    pub fn insert(self, value: MetadataValue<VE>) -> &'a mut MetadataValue<VE> {
        MetadataValue::unchecked_from_mut_header_value_ref(self.inner.insert(value.inner))
    }

    pub fn insert_entry(self, value: MetadataValue<VE>) -> OccupiedEntry<'a, Ascii> {
        OccupiedEntry {
            inner: self.inner.insert_entry(value.inner),
            phantom: PhantomData,
        }
    }
}

impl<'a, VE: ValueEncoding> OccupiedEntry<'a, VE> {
    pub fn key(&self) -> &MetadataKey<VE> {
        MetadataKey::unchecked_from_header_name_ref(self.inner.key())
    }

    pub fn get(&self) -> &MetadataValue<VE> {
        MetadataValue::unchecked_from_header_value_ref(self.inner.get())
    }

    pub fn get_mut(&mut self) -> &mut MetadataValue<VE> {
        MetadataValue::unchecked_from_mut_header_value_ref(self.inner.get_mut())
    }

    pub fn into_mut(self) -> &'a mut MetadataValue<VE> {
        MetadataValue::unchecked_from_mut_header_value_ref(self.inner.into_mut())
    }

    pub fn insert(&mut self, value: MetadataValue<VE>) -> MetadataValue<VE> {
        let header_value = self.inner.insert(value.inner);
        MetadataValue::unchecked_from_header_value(header_value)
    }

    pub fn insert_mult(&mut self, value: MetadataValue<VE>) -> ValueDrain<'_, VE> {
        ValueDrain {
            inner: self.inner.insert_mult(value.inner),
            phantom: PhantomData,
        }
    }

    pub fn append(&mut self, value: MetadataValue<VE>) {
        self.inner.append(value.inner)
    }

    pub fn remove(self) -> MetadataValue<VE> {
        let value = self.inner.remove();
        MetadataValue::unchecked_from_header_value(value)
    }

    pub fn remove_entry(self) -> (MetadataKey<VE>, MetadataValue<VE>) {
        let (name, value) = self.inner.remove_entry();
        (
            MetadataKey::unchecked_from_header_name(name),
            MetadataValue::unchecked_from_header_value(value),
        )
    }

    pub fn remove_entry_mult(self) -> (MetadataKey<VE>, ValueDrain<'a, VE>) {
        let (name, value_drain) = self.inner.remove_entry_mult();
        (
            MetadataKey::unchecked_from_header_name(name),
            ValueDrain {
                inner: value_drain,
                phantom: PhantomData,
            },
        )
    }

    pub fn iter(&self) -> ValueIter<'_, VE> {
        ValueIter {
            inner: Some(self.inner.iter()),
            phantom: PhantomData,
        }
    }

    pub fn iter_mut(&mut self) -> ValueIterMut<'_, VE> {
        ValueIterMut {
            inner: self.inner.iter_mut(),
            phantom: PhantomData,
        }
    }
}

impl<'a, VE: ValueEncoding> IntoIterator for OccupiedEntry<'a, VE>
where
    VE: 'a,
{
    type Item = &'a mut MetadataValue<VE>;
    type IntoIter = ValueIterMut<'a, VE>;

    fn into_iter(self) -> ValueIterMut<'a, VE> {
        ValueIterMut {
            inner: self.inner.into_iter(),
            phantom: PhantomData,
        }
    }
}

impl<'a, 'b: 'a, VE: ValueEncoding> IntoIterator for &'b OccupiedEntry<'a, VE> {
    type Item = &'a MetadataValue<VE>;
    type IntoIter = ValueIter<'a, VE>;

    fn into_iter(self) -> ValueIter<'a, VE> {
        self.iter()
    }
}

impl<'a, 'b: 'a, VE: ValueEncoding> IntoIterator for &'b mut OccupiedEntry<'a, VE> {
    type Item = &'a mut MetadataValue<VE>;
    type IntoIter = ValueIterMut<'a, VE>;

    fn into_iter(self) -> ValueIterMut<'a, VE> {
        self.iter_mut()
    }
}

impl<'a, VE: ValueEncoding> GetAll<'a, VE> {
    pub fn iter(&self) -> ValueIter<'a, VE> {
        ValueIter {
            inner: self.inner.as_ref().map(|inner| inner.iter()),
            phantom: PhantomData,
        }
    }
}

impl<'a, VE: ValueEncoding> PartialEq for GetAll<'a, VE> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.iter().eq(other.inner.iter())
    }
}

impl<'a, VE: ValueEncoding> IntoIterator for GetAll<'a, VE>
where
    VE: 'a,
{
    type Item = &'a MetadataValue<VE>;
    type IntoIter = ValueIter<'a, VE>;

    fn into_iter(self) -> ValueIter<'a, VE> {
        ValueIter {
            inner: self.inner.map(|inner| inner.into_iter()),
            phantom: PhantomData,
        }
    }
}

impl<'a, 'b: 'a, VE: ValueEncoding> IntoIterator for &'b GetAll<'a, VE> {
    type Item = &'a MetadataValue<VE>;
    type IntoIter = ValueIter<'a, VE>;

    fn into_iter(self) -> ValueIter<'a, VE> {
        ValueIter {
            inner: (&self.inner).as_ref().map(|inner| inner.into_iter()),
            phantom: PhantomData,
        }
    }
}

mod into_metadata_key {
    use super::{MetadataMap, MetadataValue, ValueEncoding};
    use crate::metadata::key::MetadataKey;

    pub trait IntoMetadataKey<VE: ValueEncoding>: Sealed<VE> {}

    pub trait Sealed<VE: ValueEncoding> {
        #[doc(hidden)]
        fn insert(self, map: &mut MetadataMap, val: MetadataValue<VE>)
            -> Option<MetadataValue<VE>>;

        #[doc(hidden)]
        fn append(self, map: &mut MetadataMap, val: MetadataValue<VE>) -> bool;
    }

    impl<VE: ValueEncoding> Sealed<VE> for MetadataKey<VE> {
        #[doc(hidden)]
        #[inline]
        fn insert(
            self,
            map: &mut MetadataMap,
            val: MetadataValue<VE>,
        ) -> Option<MetadataValue<VE>> {
            map.headers
                .insert(self.inner, val.inner)
                .map(&MetadataValue::unchecked_from_header_value)
        }

        #[doc(hidden)]
        #[inline]
        fn append(self, map: &mut MetadataMap, val: MetadataValue<VE>) -> bool {
            map.headers.append(self.inner, val.inner)
        }
    }

    impl<VE: ValueEncoding> IntoMetadataKey<VE> for MetadataKey<VE> {}

    impl<'a, VE: ValueEncoding> Sealed<VE> for &'a MetadataKey<VE> {
        #[doc(hidden)]
        #[inline]
        fn insert(
            self,
            map: &mut MetadataMap,
            val: MetadataValue<VE>,
        ) -> Option<MetadataValue<VE>> {
            map.headers
                .insert(&self.inner, val.inner)
                .map(&MetadataValue::unchecked_from_header_value)
        }
        #[doc(hidden)]
        #[inline]
        fn append(self, map: &mut MetadataMap, val: MetadataValue<VE>) -> bool {
            map.headers.append(&self.inner, val.inner)
        }
    }

    impl<'a, VE: ValueEncoding> IntoMetadataKey<VE> for &'a MetadataKey<VE> {}

    impl<VE: ValueEncoding> Sealed<VE> for &'static str {
        #[doc(hidden)]
        #[inline]
        fn insert(
            self,
            map: &mut MetadataMap,
            val: MetadataValue<VE>,
        ) -> Option<MetadataValue<VE>> {
            let key = MetadataKey::<VE>::from_static(self);

            map.headers
                .insert(key.inner, val.inner)
                .map(&MetadataValue::unchecked_from_header_value)
        }
        #[doc(hidden)]
        #[inline]
        fn append(self, map: &mut MetadataMap, val: MetadataValue<VE>) -> bool {
            let key = MetadataKey::<VE>::from_static(self);

            map.headers.append(key.inner, val.inner)
        }
    }

    impl<VE: ValueEncoding> IntoMetadataKey<VE> for &'static str {}
}

mod as_metadata_key {
    use super::{MetadataMap, MetadataValue, ValueEncoding};
    use crate::metadata::key::{InvalidMetadataKey, MetadataKey};
    use http::header::{Entry, GetAll, HeaderValue};

    pub trait AsMetadataKey<VE: ValueEncoding>: Sealed<VE> {}

    pub trait Sealed<VE: ValueEncoding> {
        #[doc(hidden)]
        fn get(self, map: &MetadataMap) -> Option<&MetadataValue<VE>>;

        #[doc(hidden)]
        fn get_mut(self, map: &mut MetadataMap) -> Option<&mut MetadataValue<VE>>;

        #[doc(hidden)]
        fn get_all(self, map: &MetadataMap) -> Option<GetAll<'_, HeaderValue>>;

        #[doc(hidden)]
        fn entry(self, map: &mut MetadataMap)
            -> Result<Entry<'_, HeaderValue>, InvalidMetadataKey>;

        #[doc(hidden)]
        fn remove(self, map: &mut MetadataMap) -> Option<MetadataValue<VE>>;
    }

    impl<VE: ValueEncoding> Sealed<VE> for MetadataKey<VE> {
        #[doc(hidden)]
        #[inline]
        fn get(self, map: &MetadataMap) -> Option<&MetadataValue<VE>> {
            map.headers
                .get(self.inner)
                .map(&MetadataValue::unchecked_from_header_value_ref)
        }

        #[doc(hidden)]
        #[inline]
        fn get_mut(self, map: &mut MetadataMap) -> Option<&mut MetadataValue<VE>> {
            map.headers
                .get_mut(self.inner)
                .map(&MetadataValue::unchecked_from_mut_header_value_ref)
        }

        #[doc(hidden)]
        #[inline]
        fn get_all(self, map: &MetadataMap) -> Option<GetAll<'_, HeaderValue>> {
            Some(map.headers.get_all(self.inner))
        }

        #[doc(hidden)]
        #[inline]
        fn entry(
            self,
            map: &mut MetadataMap,
        ) -> Result<Entry<'_, HeaderValue>, InvalidMetadataKey> {
            Ok(map.headers.entry(self.inner))
        }

        #[doc(hidden)]
        #[inline]
        fn remove(self, map: &mut MetadataMap) -> Option<MetadataValue<VE>> {
            map.headers
                .remove(self.inner)
                .map(&MetadataValue::unchecked_from_header_value)
        }
    }

    impl<VE: ValueEncoding> AsMetadataKey<VE> for MetadataKey<VE> {}

    impl<'a, VE: ValueEncoding> Sealed<VE> for &'a MetadataKey<VE> {
        #[doc(hidden)]
        #[inline]
        fn get(self, map: &MetadataMap) -> Option<&MetadataValue<VE>> {
            map.headers
                .get(&self.inner)
                .map(&MetadataValue::unchecked_from_header_value_ref)
        }

        #[doc(hidden)]
        #[inline]
        fn get_mut(self, map: &mut MetadataMap) -> Option<&mut MetadataValue<VE>> {
            map.headers
                .get_mut(&self.inner)
                .map(&MetadataValue::unchecked_from_mut_header_value_ref)
        }

        #[doc(hidden)]
        #[inline]
        fn get_all(self, map: &MetadataMap) -> Option<GetAll<'_, HeaderValue>> {
            Some(map.headers.get_all(&self.inner))
        }

        #[doc(hidden)]
        #[inline]
        fn entry(
            self,
            map: &mut MetadataMap,
        ) -> Result<Entry<'_, HeaderValue>, InvalidMetadataKey> {
            Ok(map.headers.entry(&self.inner))
        }

        #[doc(hidden)]
        #[inline]
        fn remove(self, map: &mut MetadataMap) -> Option<MetadataValue<VE>> {
            map.headers
                .remove(&self.inner)
                .map(&MetadataValue::unchecked_from_header_value)
        }
    }

    impl<'a, VE: ValueEncoding> AsMetadataKey<VE> for &'a MetadataKey<VE> {}

    impl<'a, VE: ValueEncoding> Sealed<VE> for &'a str {
        #[doc(hidden)]
        #[inline]
        fn get(self, map: &MetadataMap) -> Option<&MetadataValue<VE>> {
            if !VE::is_valid_key(self) {
                return None;
            }
            map.headers
                .get(self)
                .map(&MetadataValue::unchecked_from_header_value_ref)
        }

        #[doc(hidden)]
        #[inline]
        fn get_mut(self, map: &mut MetadataMap) -> Option<&mut MetadataValue<VE>> {
            if !VE::is_valid_key(self) {
                return None;
            }
            map.headers
                .get_mut(self)
                .map(&MetadataValue::unchecked_from_mut_header_value_ref)
        }

        #[doc(hidden)]
        #[inline]
        fn get_all(self, map: &MetadataMap) -> Option<GetAll<'_, HeaderValue>> {
            if !VE::is_valid_key(self) {
                return None;
            }
            Some(map.headers.get_all(self))
        }

        #[doc(hidden)]
        #[inline]
        fn entry(
            self,
            map: &mut MetadataMap,
        ) -> Result<Entry<'_, HeaderValue>, InvalidMetadataKey> {
            if !VE::is_valid_key(self) {
                return Err(InvalidMetadataKey::new());
            }

            let key = http::header::HeaderName::from_bytes(self.as_bytes())
                .map_err(|_| InvalidMetadataKey::new())?;
            let entry = map.headers.entry(key);
            Ok(entry)
        }

        #[doc(hidden)]
        #[inline]
        fn remove(self, map: &mut MetadataMap) -> Option<MetadataValue<VE>> {
            if !VE::is_valid_key(self) {
                return None;
            }
            map.headers
                .remove(self)
                .map(&MetadataValue::unchecked_from_header_value)
        }
    }

    impl<'a, VE: ValueEncoding> AsMetadataKey<VE> for &'a str {}

    impl<VE: ValueEncoding> Sealed<VE> for String {
        #[doc(hidden)]
        #[inline]
        fn get(self, map: &MetadataMap) -> Option<&MetadataValue<VE>> {
            if !VE::is_valid_key(self.as_str()) {
                return None;
            }
            map.headers
                .get(self.as_str())
                .map(&MetadataValue::unchecked_from_header_value_ref)
        }

        #[doc(hidden)]
        #[inline]
        fn get_mut(self, map: &mut MetadataMap) -> Option<&mut MetadataValue<VE>> {
            if !VE::is_valid_key(self.as_str()) {
                return None;
            }
            map.headers
                .get_mut(self.as_str())
                .map(&MetadataValue::unchecked_from_mut_header_value_ref)
        }

        #[doc(hidden)]
        #[inline]
        fn get_all(self, map: &MetadataMap) -> Option<GetAll<'_, HeaderValue>> {
            if !VE::is_valid_key(self.as_str()) {
                return None;
            }
            Some(map.headers.get_all(self.as_str()))
        }

        #[doc(hidden)]
        #[inline]
        fn entry(
            self,
            map: &mut MetadataMap,
        ) -> Result<Entry<'_, HeaderValue>, InvalidMetadataKey> {
            if !VE::is_valid_key(self.as_str()) {
                return Err(InvalidMetadataKey::new());
            }

            let key = http::header::HeaderName::from_bytes(self.as_bytes())
                .map_err(|_| InvalidMetadataKey::new())?;
            Ok(map.headers.entry(key))
        }

        #[doc(hidden)]
        #[inline]
        fn remove(self, map: &mut MetadataMap) -> Option<MetadataValue<VE>> {
            if !VE::is_valid_key(self.as_str()) {
                return None;
            }
            map.headers
                .remove(self.as_str())
                .map(&MetadataValue::unchecked_from_header_value)
        }
    }

    impl<VE: ValueEncoding> AsMetadataKey<VE> for String {}

    impl<'a, VE: ValueEncoding> Sealed<VE> for &'a String {
        #[doc(hidden)]
        #[inline]
        fn get(self, map: &MetadataMap) -> Option<&MetadataValue<VE>> {
            if !VE::is_valid_key(self) {
                return None;
            }
            map.headers
                .get(self.as_str())
                .map(&MetadataValue::unchecked_from_header_value_ref)
        }

        #[doc(hidden)]
        #[inline]
        fn get_mut(self, map: &mut MetadataMap) -> Option<&mut MetadataValue<VE>> {
            if !VE::is_valid_key(self) {
                return None;
            }
            map.headers
                .get_mut(self.as_str())
                .map(&MetadataValue::unchecked_from_mut_header_value_ref)
        }

        #[doc(hidden)]
        #[inline]
        fn get_all(self, map: &MetadataMap) -> Option<GetAll<'_, HeaderValue>> {
            if !VE::is_valid_key(self) {
                return None;
            }
            Some(map.headers.get_all(self.as_str()))
        }

        #[doc(hidden)]
        #[inline]
        fn entry(
            self,
            map: &mut MetadataMap,
        ) -> Result<Entry<'_, HeaderValue>, InvalidMetadataKey> {
            if !VE::is_valid_key(self) {
                return Err(InvalidMetadataKey::new());
            }

            let key = http::header::HeaderName::from_bytes(self.as_bytes())
                .map_err(|_| InvalidMetadataKey::new())?;
            Ok(map.headers.entry(key))
        }

        #[doc(hidden)]
        #[inline]
        fn remove(self, map: &mut MetadataMap) -> Option<MetadataValue<VE>> {
            if !VE::is_valid_key(self) {
                return None;
            }
            map.headers
                .remove(self.as_str())
                .map(&MetadataValue::unchecked_from_header_value)
        }
    }

    impl<'a, VE: ValueEncoding> AsMetadataKey<VE> for &'a String {}
}

mod as_encoding_agnostic_metadata_key {
    use super::{MetadataMap, ValueEncoding};
    use crate::metadata::key::MetadataKey;

    pub trait AsEncodingAgnosticMetadataKey: Sealed {}

    pub trait Sealed {
        #[doc(hidden)]
        fn contains_key(&self, map: &MetadataMap) -> bool;
    }

    impl<VE: ValueEncoding> Sealed for MetadataKey<VE> {
        #[doc(hidden)]
        #[inline]
        fn contains_key(&self, map: &MetadataMap) -> bool {
            map.headers.contains_key(&self.inner)
        }
    }

    impl<VE: ValueEncoding> AsEncodingAgnosticMetadataKey for MetadataKey<VE> {}

    impl<'a, VE: ValueEncoding> Sealed for &'a MetadataKey<VE> {
        #[doc(hidden)]
        #[inline]
        fn contains_key(&self, map: &MetadataMap) -> bool {
            map.headers.contains_key(&self.inner)
        }
    }

    impl<'a, VE: ValueEncoding> AsEncodingAgnosticMetadataKey for &'a MetadataKey<VE> {}

    impl<'a> Sealed for &'a str {
        #[doc(hidden)]
        #[inline]
        fn contains_key(&self, map: &MetadataMap) -> bool {
            map.headers.contains_key(*self)
        }
    }

    impl<'a> AsEncodingAgnosticMetadataKey for &'a str {}

    impl Sealed for String {
        #[doc(hidden)]
        #[inline]
        fn contains_key(&self, map: &MetadataMap) -> bool {
            map.headers.contains_key(self.as_str())
        }
    }

    impl AsEncodingAgnosticMetadataKey for String {}

    impl<'a> Sealed for &'a String {
        #[doc(hidden)]
        #[inline]
        fn contains_key(&self, map: &MetadataMap) -> bool {
            map.headers.contains_key(self.as_str())
        }
    }

    impl<'a> AsEncodingAgnosticMetadataKey for &'a String {}
}

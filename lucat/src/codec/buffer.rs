use bytes::buf::UninitSlice;
use bytes::{Buf, BufMut, BytesMut};

#[derive(Debug)]
pub struct DecodeBuf<'a> {
    buf: &'a mut BytesMut,
    len: usize,
}

#[derive(Debug)]
pub struct EncodeBuf<'a> {
    buf: &'a mut BytesMut,
}

impl<'a> DecodeBuf<'a> {
    #[allow(dead_code)]
    pub(crate) fn new(buf: &'a mut BytesMut, len: usize) -> Self {
        DecodeBuf { buf, len }
    }
}

impl Buf for DecodeBuf<'_> {
    #[inline]
    fn remaining(&self) -> usize {
        self.len
    }

    #[inline]
    fn chunk(&self) -> &[u8] {
        let ret = self.buf.chunk();

        if ret.len() > self.len {
            &ret[..self.len]
        } else {
            ret
        }
    }

    #[inline]
    fn advance(&mut self, cnt: usize) {
        assert!(cnt <= self.len);
        self.buf.advance(cnt);
        self.len -= cnt;
    }
}

impl<'a> EncodeBuf<'a> {
    #[allow(dead_code)]
    pub(crate) fn new(buf: &'a mut BytesMut) -> Self {
        EncodeBuf { buf }
    }
}

impl EncodeBuf<'_> {
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.buf.reserve(additional);
    }
}

unsafe impl BufMut for EncodeBuf<'_> {
    #[inline]
    fn remaining_mut(&self) -> usize {
        self.buf.remaining_mut()
    }

    #[inline]
    unsafe fn advance_mut(&mut self, cnt: usize) {
        self.buf.advance_mut(cnt)
    }

    #[inline]
    fn chunk_mut(&mut self) -> &mut UninitSlice {
        self.buf.chunk_mut()
    }
}

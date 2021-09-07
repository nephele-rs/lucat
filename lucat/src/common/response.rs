use crate::{metadata::MetadataMap};

#[derive(Debug)]
pub struct Response<T> {
    metadata: MetadataMap,
    payload: T,
}

impl<T> Response<T> {
    pub fn new(payload: T) -> Self {
        Response {
            metadata: MetadataMap::new(),
            payload,
        }
    }

    pub fn get_ref(&self) -> &T {
        &self.payload
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.payload
    }

    pub fn metadata(&self) -> &MetadataMap {
        &self.metadata
    }

    pub fn metadata_mut(&mut self) -> &mut MetadataMap {
        &mut self.metadata
    }

    pub fn into_inner(self) -> T {
        self.payload
    }

    pub fn into_parts(self) -> (MetadataMap, T) {
        (self.metadata, self.payload)
    }

    pub fn from_parts(metadata: MetadataMap, payload: T) -> Self {
        Self {
            metadata,
            payload,
        }
    }

    pub fn from_http(res: http::Response<T>) -> Self {
        let (head, payload) = res.into_parts();
        Response {
            metadata: MetadataMap::from_headers(head.headers),
            payload,
        }
    }

    pub fn into_http(self) -> http::Response<T> {
        let mut res = http::Response::new(self.payload);

        *res.version_mut() = http::Version::HTTP_2;
        *res.headers_mut() = self.metadata.into_sanitized_headers();

        res
    }

    pub fn map<F, U>(self, f: F) -> Response<U>
    where
        F: FnOnce(T) -> U,
    {
        let payload = f(self.payload);
        Response {
            metadata: self.metadata,
            payload,
        }
    }
}
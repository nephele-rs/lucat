use crate::metadata::{MetadataMap};

#[derive(Debug)]
pub struct Request<T> {
    metadata: MetadataMap,
    payload: T,
}

impl<T> Request<T> {
    pub fn new(payload: T) -> Self {
        Request {
            metadata: MetadataMap::new(),
            payload
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

    pub fn from_parts(parts: http::request::Parts, payload: T) -> Self {
        Self {
            metadata: MetadataMap::from_headers(parts.headers),
            payload,
        }
    } 

    pub fn from_http(http: http::Request<T>) -> Self {
        let (parts, payload) = http.into_parts();
        Request::from_parts(parts, payload)
    }

    pub fn into_http(
        self,
        uri: http::Uri,
        sanitize_headers: SanitizeHeaders,
    ) -> http::Request<T> {
        let mut request = http::Request::new(self.payload);

        *request.version_mut() = http::Version::HTTP_2;
        *request.method_mut() = http::Method::POST;
        *request.uri_mut() = uri;
        *request.headers_mut() = match sanitize_headers {
            SanitizeHeaders::Yes => self.metadata.into_sanitized_headers(),
            SanitizeHeaders::No => self.metadata.into_headers(),
        };

        request
    }

    pub fn map<F, U>(self, f: F) -> Request<U>
    where
        F: FnOnce(T) -> U,
    {
        let payload = f(self.payload);

        Request {
            metadata: self.metadata,
            payload,
        }
    }
}

impl<T> sealed::Sealed for T {}

mod sealed {
    pub trait Sealed {}
}

pub trait IntoRequest<T>: sealed::Sealed {
    fn into_request(self) -> Request<T>;
}

impl<T> IntoRequest<T> for T {
    fn into_request(self) -> Request<Self> {
        Request::new(self)
    }
}

impl<T> IntoRequest<T> for Request<T> {
    fn into_request(self) -> Request<T> {
        self
    }
}

pub enum SanitizeHeaders {
    Yes,
    No,
}
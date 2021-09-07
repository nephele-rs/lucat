use crate::metadata::MetadataMap;
use bytes::Bytes;
use http::header::{HeaderMap, HeaderValue};
use percent_encoding::{percent_decode, percent_encode, AsciiSet, CONTROLS};
use std::{borrow::Cow, error::Error, fmt};
use tracing::{debug, trace, warn};

pub const ENCODING_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'<')
    .add(b'>')
    .add(b'`')
    .add(b'?')
    .add(b'{')
    .add(b'}');

const GRPC_STATUS_HEADER_CODE: &str = "grpc-status";
const GRPC_STATUS_MESSAGE_HEADER: &str = "grpc-message";
const GRPC_STATUS_DETAILS_HEADER: &str = "grpc-status-details-bin";

pub struct Status {
    code: Code,
    message: String,
    details: Bytes,
    metadata: MetadataMap,
    source: Option<Box<dyn Error + Send + Sync + 'static>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Code {
    Ok = 0,
    Cancelled = 1,
    Unknown = 2,
    InvalidArgument = 3,
    DeadlineExceeded = 4,
    NotFound = 5,
    AlreadyExists = 6,
    PermissionDenied = 7,
    ResourceExhausted = 8,
    FailedPrecondition = 9,
    Aborted = 10,
    OutOfRange = 11,
    Unimplemented = 12,
    Internal = 13,
    Unavailable = 14,
    DataLoss = 15,
    Unauthenticated = 16,
}

impl Code {
    pub fn description(&self) -> &'static str {
        match self {
            Code::Ok => "The operation completed successfully",
            Code::Cancelled => "The operation was cancelled",
            Code::Unknown => "Unknown error",
            Code::InvalidArgument => "Client specified an invalid argument",
            Code::DeadlineExceeded => "Deadline expired before operation could complete",
            Code::NotFound => "Some requested entity was not found",
            Code::AlreadyExists => "Some entity that we attempted to create already exists",
            Code::PermissionDenied => {
                "The caller does not have permission to execute the specified operation"
            }
            Code::ResourceExhausted => "Some resource has been exhausted",
            Code::FailedPrecondition => {
                "The system is not in a state required for the operation's execution"
            }
            Code::Aborted => "The operation was aborted",
            Code::OutOfRange => "Operation was attempted past the valid range",
            Code::Unimplemented => "Operation is not implemented or not supported",
            Code::Internal => "Internal error",
            Code::Unavailable => "The service is currently unavailable",
            Code::DataLoss => "Unrecoverable data loss or corruption",
            Code::Unauthenticated => "The request does not have valid authentication credentials",
        }
    }
}

impl std::fmt::Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.description(), f)
    }
}

impl Status {
    pub fn new(code: Code, message: impl Into<String>) -> Status {
        Status {
            code,
            message: message.into(),
            details: Bytes::new(),
            metadata: MetadataMap::new(),
            source: None,
        }
    }

    pub fn ok(message: impl Into<String>) -> Status {
        Status::new(Code::Ok, message)
    }

    pub fn cancelled(message: impl Into<String>) -> Status {
        Status::new(Code::Cancelled, message)
    }

    pub fn unknown(message: impl Into<String>) -> Status {
        Status::new(Code::Unknown, message)
    }

    pub fn invalid_argument(message: impl Into<String>) -> Status {
        Status::new(Code::InvalidArgument, message)
    }

    pub fn deadline_exceeded(message: impl Into<String>) -> Status {
        Status::new(Code::DeadlineExceeded, message)
    }

    pub fn not_found(message: impl Into<String>) -> Status {
        Status::new(Code::NotFound, message)
    }

    pub fn already_exists(message: impl Into<String>) -> Status {
        Status::new(Code::AlreadyExists, message)
    }

    pub fn permission_denied(message: impl Into<String>) -> Status {
        Status::new(Code::PermissionDenied, message)
    }

    pub fn resource_exhausted(message: impl Into<String>) -> Status {
        Status::new(Code::ResourceExhausted, message)
    }

    pub fn failed_precondition(message: impl Into<String>) -> Status {
        Status::new(Code::FailedPrecondition, message)
    }

    pub fn aborted(message: impl Into<String>) -> Status {
        Status::new(Code::Aborted, message)
    }

    pub fn out_of_range(message: impl Into<String>) -> Status {
        Status::new(Code::OutOfRange, message)
    }

    pub fn unimplemented(message: impl Into<String>) -> Status {
        Status::new(Code::Unimplemented, message)
    }

    pub fn internal(message: impl Into<String>) -> Status {
        Status::new(Code::Internal, message)
    }

    pub fn unavailable(message: impl Into<String>) -> Status {
        Status::new(Code::Unavailable, message)
    }

    pub fn data_loss(message: impl Into<String>) -> Status {
        Status::new(Code::DataLoss, message)
    }

    pub fn unauthenticated(message: impl Into<String>) -> Status {
        Status::new(Code::Unauthenticated, message)
    }

    #[cfg_attr(not(feature = "transport"), allow(dead_code))]
    pub(crate) fn from_error(err: Box<dyn Error + Send + Sync + 'static>) -> Status {
        Status::try_from_error(err)
            .unwrap_or_else(|err| Status::new(Code::Unknown, err.to_string()))
    }

    pub(crate) fn try_from_error(
        err: Box<dyn Error + Send + Sync + 'static>,
    ) -> Result<Status, Box<dyn Error + Send + Sync + 'static>> {
        let err = match err.downcast::<Status>() {
            Ok(status) => {
                return Ok(*status);
            }
            Err(err) => err,
        };

        #[cfg(feature = "transport")]
        let err = match err.downcast::<h2::Error>() {
            Ok(h2) => {
                return Ok(Status::from_h2_error(&*h2));
            }
            Err(err) => err,
        };

        if let Some(status) = find_status_in_source_chain(&*err) {
            return Ok(status);
        }

        Err(err)
    }

    #[cfg(feature = "transport")]
    fn from_h2_error(err: &h2::Error) -> Status {
        // See https://github.com/grpc/grpc/blob/3977c30/doc/PROTOCOL-HTTP2.md#errors
        let code = match err.reason() {
            Some(h2::Reason::NO_ERROR)
            | Some(h2::Reason::PROTOCOL_ERROR)
            | Some(h2::Reason::INTERNAL_ERROR)
            | Some(h2::Reason::FLOW_CONTROL_ERROR)
            | Some(h2::Reason::SETTINGS_TIMEOUT)
            | Some(h2::Reason::COMPRESSION_ERROR)
            | Some(h2::Reason::CONNECT_ERROR) => Code::Internal,
            Some(h2::Reason::REFUSED_STREAM) => Code::Unavailable,
            Some(h2::Reason::CANCEL) => Code::Cancelled,
            Some(h2::Reason::ENHANCE_YOUR_CALM) => Code::ResourceExhausted,
            Some(h2::Reason::INADEQUATE_SECURITY) => Code::PermissionDenied,

            _ => Code::Unknown,
        };

        let mut status = Self::new(code, format!("h2 protocol error: {}", err));
        let error = err
            .reason()
            .map(h2::Error::from)
            .map(|err| Box::new(err) as Box<dyn Error + Send + Sync + 'static>);
        status.source = error;
        status
    }

    #[cfg(feature = "transport")]
    fn to_h2_error(&self) -> h2::Error {
        let reason = match self.code {
            Code::Cancelled => h2::Reason::CANCEL,
            _ => h2::Reason::INTERNAL_ERROR,
        };

        reason.into()
    }

    pub fn map_error<E>(err: E) -> Status
    where
        E: Into<Box<dyn Error + Send + Sync>>,
    {
        let err: Box<dyn Error + Send + Sync> = err.into();
        Status::from_error(err)
    }

    pub fn from_header_map(header_map: &HeaderMap) -> Option<Status> {
        header_map.get(GRPC_STATUS_HEADER_CODE).map(|code| {
            let code = Code::from_bytes(code.as_ref());
            let error_message = header_map
                .get(GRPC_STATUS_MESSAGE_HEADER)
                .map(|header| {
                    percent_decode(header.as_bytes())
                        .decode_utf8()
                        .map(|cow| cow.to_string())
                })
                .unwrap_or_else(|| Ok(String::new()));

            let details = header_map
                .get(GRPC_STATUS_DETAILS_HEADER)
                .map(|h| {
                    base64::decode(h.as_bytes())
                        .expect("Invalid status header, expected base64 encoded value")
                })
                .map(Bytes::from)
                .unwrap_or_else(Bytes::new);

            let mut other_headers = header_map.clone();
            other_headers.remove(GRPC_STATUS_HEADER_CODE);
            other_headers.remove(GRPC_STATUS_MESSAGE_HEADER);
            other_headers.remove(GRPC_STATUS_DETAILS_HEADER);

            match error_message {
                Ok(message) => Status {
                    code,
                    message,
                    details,
                    metadata: MetadataMap::from_headers(other_headers),
                    source: None,
                },
                Err(err) => {
                    warn!("Error deserializing status message header: {}", err);
                    Status {
                        code: Code::Unknown,
                        message: format!("Error deserializing status message header: {}", err),
                        details,
                        metadata: MetadataMap::from_headers(other_headers),
                        source: None,
                    }
                }
            }
        })
    }

    pub fn code(&self) -> Code {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn details(&self) -> &[u8] {
        &self.details
    }

    pub fn metadata(&self) -> &MetadataMap {
        &self.metadata
    }

    pub fn metadata_mut(&mut self) -> &mut MetadataMap {
        &mut self.metadata
    }

    pub fn to_header_map(&self) -> Result<HeaderMap, Self> {
        let mut header_map = HeaderMap::with_capacity(3 + self.metadata.len());
        self.add_header(&mut header_map)?;
        Ok(header_map)
    }

    pub fn add_header(&self, header_map: &mut HeaderMap) -> Result<(), Self> {
        header_map.extend(self.metadata.clone().into_sanitized_headers());

        header_map.insert(GRPC_STATUS_HEADER_CODE, self.code.to_header_value());

        if !self.message.is_empty() {
            let to_write = Bytes::copy_from_slice(
                Cow::from(percent_encode(&self.message().as_bytes(), ENCODING_SET)).as_bytes(),
            );

            header_map.insert(
                GRPC_STATUS_MESSAGE_HEADER,
                HeaderValue::from_maybe_shared(to_write).map_err(invalid_header_value_byte)?,
            );
        }

        if !self.details.is_empty() {
            let details = base64::encode_config(&self.details[..], base64::STANDARD_NO_PAD);

            header_map.insert(
                GRPC_STATUS_DETAILS_HEADER,
                HeaderValue::from_maybe_shared(details).map_err(invalid_header_value_byte)?,
            );
        }

        Ok(())
    }

    pub fn with_details(code: Code, message: impl Into<String>, details: Bytes) -> Status {
        Self::with_details_and_metadata(code, message, details, MetadataMap::new())
    }

    pub fn with_metadata(code: Code, message: impl Into<String>, metadata: MetadataMap) -> Status {
        Self::with_details_and_metadata(code, message, Bytes::new(), metadata)
    }

    pub fn with_details_and_metadata(
        code: Code,
        message: impl Into<String>,
        details: Bytes,
        metadata: MetadataMap,
    ) -> Status {
        Status {
            code,
            message: message.into(),
            details,
            metadata,
            source: None,
        }
    }
}

fn find_status_in_source_chain(err: &(dyn Error + 'static)) -> Option<Status> {
    let mut source = Some(err);

    while let Some(err) = source {
        if let Some(status) = err.downcast_ref::<Status>() {
            return Some(Status {
                code: status.code,
                message: status.message.clone(),
                details: status.details.clone(),
                metadata: status.metadata.clone(),
                source: None,
            });
        }

        #[cfg(feature = "transport")]
        if let Some(timeout) = err.downcast_ref::<crate::transport::TimeoutExpired>() {
            return Some(Status::cancelled(timeout.to_string()));
        }

        source = err.source();
    }

    None
}
impl fmt::Debug for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_struct("Status");

        builder.field("code", &self.code);

        if !self.message.is_empty() {
            builder.field("message", &self.message);
        }

        if !self.details.is_empty() {
            builder.field("details", &self.details);
        }

        if !self.metadata.is_empty() {
            builder.field("metadata", &self.metadata);
        }

        builder.field("source", &self.source);

        builder.finish()
    }
}

pub fn invalid_header_value_byte<Error: fmt::Display>(err: Error) -> Status {
    debug!("Invalid header: {}", err);
    Status::new(
        Code::Internal,
        "Couldn't serialize non-text grpc status header".to_string(),
    )
}

#[cfg(feature = "transport")]
impl From<h2::Error> for Status {
    fn from(err: h2::Error) -> Self {
        Status::from_h2_error(&err)
    }
}

#[cfg(feature = "transport")]
impl From<Status> for h2::Error {
    fn from(status: Status) -> Self {
        status.to_h2_error()
    }
}

impl From<std::io::Error> for Status {
    fn from(err: std::io::Error) -> Self {
        use std::io::ErrorKind;
        let code = match err.kind() {
            ErrorKind::BrokenPipe
            | ErrorKind::WouldBlock
            | ErrorKind::WriteZero
            | ErrorKind::Interrupted => Code::Internal,
            ErrorKind::ConnectionRefused
            | ErrorKind::ConnectionReset
            | ErrorKind::NotConnected
            | ErrorKind::AddrInUse
            | ErrorKind::AddrNotAvailable => Code::Unavailable,
            ErrorKind::AlreadyExists => Code::AlreadyExists,
            ErrorKind::ConnectionAborted => Code::Aborted,
            ErrorKind::InvalidData => Code::DataLoss,
            ErrorKind::InvalidInput => Code::InvalidArgument,
            ErrorKind::NotFound => Code::NotFound,
            ErrorKind::PermissionDenied => Code::PermissionDenied,
            ErrorKind::TimedOut => Code::DeadlineExceeded,
            ErrorKind::UnexpectedEof => Code::OutOfRange,
            _ => Code::Unknown,
        };
        Status::new(code, err.to_string())
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "status: {:?}, message: {:?}, details: {:?}, metadata: {:?}",
            self.code(),
            self.message(),
            self.details(),
            self.metadata(),
        )
    }
}

impl Error for Status {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|err| (&**err) as _)
    }
}

pub fn infer_grpc_status(
    trailers: Option<&HeaderMap>,
    status_code: http::StatusCode,
) -> Result<(), Option<Status>> {
    if let Some(trailers) = trailers {
        if let Some(status) = Status::from_header_map(&trailers) {
            if status.code() == Code::Ok {
                return Ok(());
            } else {
                return Err(status.into());
            }
        }
    }
    trace!("trailers missing grpc-status");
    let code = match status_code {
        http::StatusCode::BAD_REQUEST => Code::Internal,
        http::StatusCode::UNAUTHORIZED => Code::Unauthenticated,
        http::StatusCode::FORBIDDEN => Code::PermissionDenied,
        http::StatusCode::NOT_FOUND => Code::Unimplemented,
        http::StatusCode::TOO_MANY_REQUESTS
        | http::StatusCode::BAD_GATEWAY
        | http::StatusCode::SERVICE_UNAVAILABLE
        | http::StatusCode::GATEWAY_TIMEOUT => Code::Unavailable,

        http::StatusCode::OK => return Err(None),
        _ => Code::Unknown,
    };

    let msg = format!(
        "grpc-status header missing, mapped from HTTP status code {}",
        status_code.as_u16(),
    );
    let status = Status::new(code, msg);
    Err(status.into())
}

impl Code {
    pub fn from_i32(i: i32) -> Code {
        Code::from(i)
    }

    pub fn from_bytes(bytes: &[u8]) -> Code {
        match bytes.len() {
            1 => match bytes[0] {
                b'0' => Code::Ok,
                b'1' => Code::Cancelled,
                b'2' => Code::Unknown,
                b'3' => Code::InvalidArgument,
                b'4' => Code::DeadlineExceeded,
                b'5' => Code::NotFound,
                b'6' => Code::AlreadyExists,
                b'7' => Code::PermissionDenied,
                b'8' => Code::ResourceExhausted,
                b'9' => Code::FailedPrecondition,
                _ => Code::parse_err(),
            },
            2 => match (bytes[0], bytes[1]) {
                (b'1', b'0') => Code::Aborted,
                (b'1', b'1') => Code::OutOfRange,
                (b'1', b'2') => Code::Unimplemented,
                (b'1', b'3') => Code::Internal,
                (b'1', b'4') => Code::Unavailable,
                (b'1', b'5') => Code::DataLoss,
                (b'1', b'6') => Code::Unauthenticated,
                _ => Code::parse_err(),
            },
            _ => Code::parse_err(),
        }
    }

    pub fn to_header_value(self) -> HeaderValue {
        match self {
            Code::Ok => HeaderValue::from_static("0"),
            Code::Cancelled => HeaderValue::from_static("1"),
            Code::Unknown => HeaderValue::from_static("2"),
            Code::InvalidArgument => HeaderValue::from_static("3"),
            Code::DeadlineExceeded => HeaderValue::from_static("4"),
            Code::NotFound => HeaderValue::from_static("5"),
            Code::AlreadyExists => HeaderValue::from_static("6"),
            Code::PermissionDenied => HeaderValue::from_static("7"),
            Code::ResourceExhausted => HeaderValue::from_static("8"),
            Code::FailedPrecondition => HeaderValue::from_static("9"),
            Code::Aborted => HeaderValue::from_static("10"),
            Code::OutOfRange => HeaderValue::from_static("11"),
            Code::Unimplemented => HeaderValue::from_static("12"),
            Code::Internal => HeaderValue::from_static("13"),
            Code::Unavailable => HeaderValue::from_static("14"),
            Code::DataLoss => HeaderValue::from_static("15"),
            Code::Unauthenticated => HeaderValue::from_static("16"),
        }
    }

    fn parse_err() -> Code {
        trace!("error parsing grpc-status");
        Code::Unknown
    }
}

impl From<i32> for Code {
    fn from(i: i32) -> Self {
        match i {
            0 => Code::Ok,
            1 => Code::Cancelled,
            2 => Code::Unknown,
            3 => Code::InvalidArgument,
            4 => Code::DeadlineExceeded,
            5 => Code::NotFound,
            6 => Code::AlreadyExists,
            7 => Code::PermissionDenied,
            8 => Code::ResourceExhausted,
            9 => Code::FailedPrecondition,
            10 => Code::Aborted,
            11 => Code::OutOfRange,
            12 => Code::Unimplemented,
            13 => Code::Internal,
            14 => Code::Unavailable,
            15 => Code::DataLoss,
            16 => Code::Unauthenticated,

            _ => Code::Unknown,
        }
    }
}

impl From<Code> for i32 {
    #[inline]
    fn from(code: Code) -> i32 {
        code as i32
    }
}

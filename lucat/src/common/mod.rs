pub mod request;
pub mod response;
pub mod body;
pub mod status;

pub use request::{IntoRequest, Request};
pub use response::Response;

pub use status::{Code, Status};

pub use body::Body;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug)]
pub enum Never {}

impl std::fmt::Display for Never {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {}
    }
}

impl std::error::Error for Never {}

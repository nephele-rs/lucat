pub mod codec;
pub mod common;
pub mod metadata;
pub mod transport;
pub mod controller;
pub mod runtime;
pub mod generator;
pub mod macros;

pub use async_trait::async_trait;

pub use common::status::{Code, Status};

pub use common::response::Response;

pub use common::request::Request;

pub use common::Body;

pub use controller::server;
pub use controller::client;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub use runtime::{Service, InstantService, SimpleInstantService};

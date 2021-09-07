mod service;

pub use service::{Service, InstantService, SimpleInstantService};

pub type BoxFuture<T, E> =
    std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send + 'static>>;

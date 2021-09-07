pub mod client;
pub mod server;
pub mod service;
pub mod generator;

pub use generator::{compile_protos, configure, Builder};

pub use std::io::{self, Write};
pub use std::process::{exit, Command};

pub use proc_macro2::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream};
pub use quote::TokenStreamExt;

pub use service::{Service, Method, Attributes};

pub use service::{generate_doc_comments, naive_snake_case, fmt, generate_doc_comment};

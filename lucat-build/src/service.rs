use super::TokenStream;
use quote::TokenStreamExt;
use std::io::{self, Write};
use std::process::{exit, Command};
use proc_macro2::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span};

pub trait Service {
    const CODEC_PATH: &'static str;

    type Comment: AsRef<str>;

    type Method: Method;

    fn name(&self) -> &str;
    fn package(&self) -> &str;
    fn identifier(&self) -> &str;
    fn methods(&self) -> &[Self::Method];
    fn comment(&self) -> &[Self::Comment];
}

pub trait Method {
    const CODEC_PATH: &'static str;
    type Comment: AsRef<str>;

    fn name(&self) -> &str;
    fn identifier(&self) -> &str;
    fn client_streaming(&self) -> bool;
    fn server_streaming(&self) -> bool;
    fn comment(&self) -> &[Self::Comment];
    fn request_response_name(
        &self,
        proto_path: &str,
        compile_well_known_types: bool,
    ) -> (TokenStream, TokenStream);
}

#[derive(Debug, Default, Clone)]
pub struct Attributes {
    module: Vec<(String, String)>,
    structure: Vec<(String, String)>,
}

impl Attributes {
    pub fn for_mod(&self, name: &str) -> Vec<syn::Attribute> {
        generate_attributes(name, &self.module)
    }

    pub fn for_struct(&self, name: &str) -> Vec<syn::Attribute> {
        generate_attributes(name, &self.structure)
    }

    pub fn push_mod(&mut self, pattern: impl Into<String>, attr: impl Into<String>) {
        self.module.push((pattern.into(), attr.into()));
    }

    pub fn push_struct(&mut self, pattern: impl Into<String>, attr: impl Into<String>) {
        self.structure.push((pattern.into(), attr.into()));
    }
}

fn generate_attributes<'a>(
    name: &str,
    attrs: impl IntoIterator<Item = &'a (String, String)>,
) -> Vec<syn::Attribute> {
    attrs
        .into_iter()
        .filter(|(matcher, _)| match_name(matcher, name))
        .flat_map(|(_, attr)| {
            syn::parse_str::<syn::DeriveInput>(&format!("{}\nstruct fake;", attr))
                .unwrap()
                .attrs
        })
        .collect::<Vec<_>>()
}

#[cfg(feature = "rustfmt")]
#[cfg_attr(docsrs, doc(cfg(feature = "rustfmt")))]
pub fn fmt(out_dir: &str) {
    let dir = std::fs::read_dir(out_dir).unwrap();

    for entry in dir {
        let file = entry.unwrap().file_name().into_string().unwrap();
        if !file.ends_with(".rs") {
            continue;
        }
        let result =
            Command::new(std::env::var("RUSTFMT").unwrap_or_else(|_| "rustfmt".to_owned()))
                .arg("--emit")
                .arg("files")
                .arg("--edition")
                .arg("2018")
                .arg(format!("{}/{}", out_dir, file))
                .output();

        match result {
            Err(e) => {
                eprintln!("error running rustfmt: {:?}", e);
                exit(1)
            }
            Ok(output) => {
                if !output.status.success() {
                    io::stdout().write_all(&output.stdout).unwrap();
                    io::stderr().write_all(&output.stderr).unwrap();
                    exit(output.status.code().unwrap_or(1))
                }
            }
        }
    }
}

pub fn generate_doc_comment<S: AsRef<str>>(comment: S) -> TokenStream {
    let mut doc_stream = TokenStream::new();

    doc_stream.append(Ident::new("doc", Span::call_site()));
    doc_stream.append(Punct::new('=', Spacing::Alone));
    doc_stream.append(Literal::string(comment.as_ref()));

    let group = Group::new(Delimiter::Bracket, doc_stream);

    let mut stream = TokenStream::new();
    stream.append(Punct::new('#', Spacing::Alone));
    stream.append(group);
    stream
}

pub fn generate_doc_comments<T: AsRef<str>>(comments: &[T]) -> super::TokenStream {
    let mut stream = TokenStream::new();

    for comment in comments {
        stream.extend(generate_doc_comment(comment));
    }

    stream
}

pub fn match_name(pattern: &str, path: &str) -> bool {
    if pattern.is_empty() {
        false
    } else if pattern == "." || pattern == path {
        true
    } else {
        let pattern_segments = pattern.split('.').collect::<Vec<_>>();
        let path_segments = path.split('.').collect::<Vec<_>>();

        if &pattern[..1] == "." {
            if pattern_segments.len() > path_segments.len() {
                false
            } else {
                pattern_segments[..] == path_segments[..pattern_segments.len()]
            }
        } else if pattern_segments.len() > path_segments.len() {
            false
        } else {
            pattern_segments[..] == path_segments[path_segments.len() - pattern_segments.len()..]
        }
    }
}

pub fn naive_snake_case(name: &str) -> String {
    let mut s = String::new();
    let mut it = name.chars().peekable();

    while let Some(x) = it.next() {
        s.push(x.to_ascii_lowercase());
        if let Some(y) = it.peek() {
            if y.is_uppercase() {
                s.push('_');
            }
        }
    }

    s
}

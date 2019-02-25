//! An [SDLang][sdlang] parser library.
//!
//! [SDLang][sdlang] is a simple and concise way to textually represent data.
//! It has an XML-like structure - tags, values, and attributes - which makes
//! it a versatily choice for data serialization, configuration files, or
//! declarative languages. Its syntax was inspired by the C family of languages
//! (C/C++, C#, D, Java, ...).
//!
//! Here's an example from the official website:
//! ```sdlang
//! // This is a node with a single string value
//! title "Hello, World"
//!
//! // Multiple values are supported, too
//! bookmarks 12 15 188 1234
//!
//! // Nodes can have attributes
//! author "Peter Parker" email="peter@example.org" active=true
//!
//! // Nodes can be arbitrarily nested
//! contents {
//!     section "First Section" {
//!         paragraph "This is the first paragraph"
//!         paragraph "This is the second paragraph"
//!     }
//! }
//!
//! // Anonymous nodes are supported
//! "This text is the value of an anonymous node!"
//!
//! // This makes things like matrix definiotns very convenient
//! matrix {
//!     1 0 0
//!     0 1 0
//!     0 0 1
//! }
//! ```
//!
//! Parsing is made as easy as this:
//! ```rust
//! extern crate sdlang;
//!
//! // Prints `tag hello_world: "text"`
//! println!("{}", sdlang::parse_text("hello_world \"text\"").unwrap());
//! ```
//! Note that all SDLang-related types (i.e `Tag`, `Attribute` and `Value`)
//! implement `FromStr` so that they can be used with `str::parse`. Note,
//! however, that in order to parse a whole file, which may have multiple root
//! tags, use `parse_text` or `parse_file`.
//!
//! [sdlang]: https://sdlang.org "Official SDLang Website"

// Crates
extern crate base64;
extern crate chrono;
extern crate itertools;
extern crate pest;
#[macro_use] extern crate pest_derive;

// Modules
mod grammar;
mod types;
mod parse;

// Public types
pub use grammar::{Error, ParseRes as Result};
pub use types::{Value, Attribute, Tag, Date, DateTime};

// Internal usage here
use std::{io, io::Read};

/// Reads everything from the given Reader and parses it.
///
/// Look at `parse_text` for more information.
///
/// The reader is internally buffered using `std::io::BufReader`.
pub fn parse_file<R>(data: R) -> io::Result<Result<Tag>>
where R: io::Read {
    let mut res = String::new();
    io::BufReader::new(data).read_to_string(&mut res)?;
    Ok(parse_text(res.as_str()))
}

/// Parses the given text into a root tag.
///
/// The name of the root tag is `""` (nothing); It has no namespace, values, or
/// attributes; it only has a list of child tags.
pub fn parse_text(data: &str) -> Result<Tag> {
    Ok(Tag::new(String::new())
        .tags(grammar::parse(grammar::Rule::tagtree, data)
              .and_then(parse::tagtree)?))
}

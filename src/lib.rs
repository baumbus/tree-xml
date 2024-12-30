//! Tree abstraction on top of the high performence XML reader/writer [quick-xml](https://crates.io/crates/quick-xml)

#![forbid(unsafe_code)]
// #![deny(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::style)]
#![warn(clippy::complexity)]
#![warn(clippy::cargo)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
// #![warn(clippy::restriction)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

/// Error management module.
pub mod error;
/// High level representation of an XML DOM element.
pub mod node;
/// Contains helper traits to work with [Nodes](crate::node::Node).
pub mod traits;

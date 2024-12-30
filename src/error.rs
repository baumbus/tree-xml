use thiserror::Error;

use std::str::Utf8Error;

use quick_xml::events::attributes::AttrError;
use quick_xml::Error as XmlError;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error kind: {0}")]
    IOErrorKind(std::io::ErrorKind),
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Error from the quick-xml crate: {0}")]
    XmlError(#[from] XmlError),
    #[error("UTF8 error while parsing to node: {0}")]
    Utf8Error(#[from] Utf8Error),
    #[error("Attribute error: {0}")]
    AttributeXmlError(#[from] AttrError),
    #[error("End of XML file")]
    Eof,
    #[error("Parse node error: {0}")]
    ParseNodeError(#[from] ParseNodeError),
}

#[derive(Debug, Error)]
pub enum ParseNodeError {
    #[error("Expected {0} but found {1}")]
    WrongName(String, String),
    #[error("No child <{0}> found in parent <{1}>")]
    MissingChild(String, String),
    #[error("No attribute with key '{0}' found in <{1}>")]
    MissingAttribute(String, String),
}

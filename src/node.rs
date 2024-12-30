use core::{fmt, str};
use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::io::{BufRead, Cursor, Write};
use std::str::FromStr;

#[cfg(feature = "log")]
use log::{error, info, trace, warn};
use quick_xml::events::attributes::Attribute;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::name::QName;
use quick_xml::{Reader, Writer};

use crate::error::Error;
use crate::error::ParseNodeError;
use crate::error::Result;

/// A high level tree representation of an XML DOM class.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Node {
    name: String,
    content: String,
    attributes: HashMap<String, String>,
    childs: Vec<Node>,
}

/// A builder for a [`Node`] struct.
#[derive(Debug, Default, Clone)]
pub struct NodeBuilder<'a> {
    name: &'a str,
    content: &'a str,
    attributes: HashMap<String, String>,
    childs: Vec<Node>,
}

impl Node {
    /// Gets a [`NodeBuilder`] for a [`Node`] with a specified name.
    #[must_use]
    pub fn builder(name: &str) -> NodeBuilder {
        #[cfg(feature = "log")]
        trace!("new builder for {}", name);
        NodeBuilder::new(name)
    }

    /// Gets the name of the current [`Node`].
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Gets the content of the current [`Node`].
    #[must_use]
    pub fn content(&self) -> &str {
        self.content.as_str()
    }

    /// Searches for a attribute with the specified key and return it if it is found return it as a [`prim@str`].
    ///
    /// # Errors
    ///
    /// This function will return an error if the [`Node`] has no attribute with the given key.
    pub fn attribute(&self, key: &str) -> Result<&str> {
        #[cfg(feature = "log")]
        trace!("searching for attribute '{}' in <{}>", key, self.name());
        self.attributes
            .get(key)
            .map(std::string::String::as_str)
            .ok_or_else(|| {
                ParseNodeError::MissingAttribute(key.to_owned(), self.name.clone()).into()
            })
    }

    /// Gets the childs of the [`Node`] as an [`Iterator`].
    pub fn childs(&self) -> impl Iterator<Item = &Self> {
        self.childs.iter()
    }

    /// Checks if the [`Node`] has an attribute with the given key.
    #[must_use]
    pub fn has_attribute(&self, key: &str) -> bool {
        #[cfg(feature = "log")]
        trace!("searching for attribute '{}' in <{}>", key, self.name());
        self.attributes.contains_key(key)
    }

    /// Checks if the [`Node`] has childs.
    #[must_use]
    pub fn has_childs(&self) -> bool {
        !self.childs.is_empty()
    }

    /// Gets the amount of childs the [`Node`] has.
    #[must_use]
    pub fn child_count(&self) -> usize {
        self.childs.len()
    }

    /// Searches for a child with the given name.
    ///
    /// # Errors
    ///
    /// This function will return an error if the [`Node`] has no child with the given name.
    pub fn child_by_name<'a, 'n: 'a>(&'a self, name: &'n str) -> Result<&'a Self> {
        #[cfg(feature = "log")]
        trace!("searching for child <{}> inside of <{}>", name, self.name());
        self.childs_by_name(name)
            .next()
            .ok_or_else(|| ParseNodeError::MissingChild(name.to_owned(), self.name.clone()).into())
    }

    /// Returns an iterator with all childs that have the given name.
    pub fn childs_by_name<'a, 'n: 'a>(
        &'a self,
        name: &'n str,
    ) -> impl Iterator<Item = &'a Self> + 'a {
        #[cfg(feature = "log")]
        trace!(
            "construct iterator from all childs <{}> from parent <{}>",
            name,
            self.name()
        );
        self.childs.iter().filter(move |c| c.name == name)
    }

    /// Write the XML [`Node`] as a character stream to the given [`Writer`]. Internal function only.
    ///
    /// # Errors
    ///
    /// This function will return an error if the event writing on the [`Writer`] fails.
    fn write_to_impl<W>(&self, writer: &mut Writer<W>) -> Result<()>
    where
        W: std::io::Write,
    {
        let start = BytesStart::from(self);

        if self.childs.is_empty() && self.content.is_empty() {
            writer.write_event(Event::Empty(start))?;
        } else {
            writer.write_event(Event::Start(start))?;

            if !self.content.is_empty() {
                writer.write_event(Event::Text(BytesText::new(&self.content)))?;
            }

            if !self.childs.is_empty() {
                for child in &self.childs {
                    child.write_to(writer)?;
                }
            }

            writer.write_event(Event::End(BytesEnd::new(&self.name)))?;
        }

        Ok(())
    }

    /// Write the XML [`Node`] as a character stream to the given [`Writer`].
    ///
    /// # Errors
    ///
    /// This function will return an error if the event writing on the [`Writer`] fails.
    pub fn write_to<W>(&self, writer: &mut Writer<W>) -> Result<()>
    where
        W: Write,
    {
        #[cfg(feature = "log")]
        trace!("writing <{}>", self.name());
        self.write_to_impl(writer)?;
        writer.get_mut().flush()?;

        Ok(())
    }

    /// Parses the stream from a [`Reader`] to a [`Node`]
    ///
    /// # Errors
    ///
    /// This function will return an error if the [`Reader`] gets an errous value or if the end of the stream is reached.
    pub fn read_from<R>(reader: &mut Reader<R>) -> Result<Self>
    where
        R: BufRead,
    {
        let mut node_stack = VecDeque::<Self>::new();
        let mut buf = Vec::new();

        let node = loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref start)) => {
                    #[cfg(feature = "log")]
                    trace!("Read start event");
                    let node = Self::try_from(start)?;
                    node_stack.push_back(node);
                }
                Ok(Event::Empty(ref start)) => {
                    #[cfg(feature = "log")]
                    trace!("Read empty event");
                    let node = Self::try_from(start)?;
                    if let Some(mut parent) = node_stack.pop_back() {
                        parent.childs.push(node);
                        node_stack.push_back(parent);
                    } else {
                        break Ok(node);
                    }
                }
                Ok(Event::End(ref end)) => {
                    #[cfg(feature = "log")]
                    trace!("Read end event");
                    #[cfg(not(feature = "log"))]
                    let _ = end;
                    if let Some(node) = node_stack.pop_back() {
                        if let Some(mut parent) = node_stack.pop_back() {
                            parent.childs.push(node);
                            node_stack.push_back(parent);
                        } else {
                            break Ok(node);
                        }
                    } else {
                        #[cfg(feature = "log")]
                        error!(
                            "Found closing element </{}> without an opening element before",
                            str::from_utf8(end.name().as_ref())?
                        );
                    }
                }
                Ok(Event::Text(ref t)) => {
                    #[cfg(feature = "log")]
                    trace!("Read text event");
                    let content = str::from_utf8(t)?.trim();
                    if !content.is_empty() {
                        if let Some(node) = node_stack.back_mut() {
                            node.content += content;
                        } else {
                            #[cfg(feature = "log")]
                            warn!("Found characters {} outside of any node", content);
                        }
                    }
                }
                Ok(Event::Eof) => break Err(Error::Eof),
                Err(e) => break Err(Error::from(e)),
                #[cfg(feature = "log")]
                ev => info!("Read other event: {:?}", ev),
                #[cfg(not(feature = "log"))]
                _ => {}
            }
        }?;

        Ok(node)
    }
}

impl<'a> TryFrom<&BytesStart<'a>> for Node {
    type Error = Error;

    fn try_from(value: &BytesStart<'a>) -> Result<Self> {
        Ok(Self {
            name: str::from_utf8(value.name().as_ref())?.to_owned(),
            content: String::new(),
            attributes: value
                .attributes()
                .map(|res| {
                    let attribute = res?;
                    let key = str::from_utf8(attribute.key.as_ref())?.to_owned();
                    let value = str::from_utf8(&attribute.value)?.to_owned();
                    Ok((key, value))
                })
                .collect::<Result<HashMap<_, _>>>()?,
            childs: Vec::new(),
        })
    }
}

impl<'a> From<&'a Node> for BytesStart<'a> {
    fn from(node: &'a Node) -> Self {
        BytesStart::new(&node.name).with_attributes(node.attributes.iter().map(|(k, v)| {
            Attribute {
                key: QName(k.as_bytes()),
                value: Cow::Borrowed(v.as_bytes()),
            }
        }))
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut writer = Writer::new(Cursor::new(Vec::new()));
        self.write_to(&mut writer).map_err(|_| fmt::Error)?;
        write!(
            f,
            "{}",
            str::from_utf8(&writer.into_inner().into_inner()).map_err(|_| fmt::Error)?
        )
    }
}

impl FromStr for Node {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::read_from(&mut Reader::from_str(s))
    }
}

impl<'a> NodeBuilder<'a> {
    /// Creates a builder for a [`Node`] with the name set to the givin parameter.
    #[must_use]
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            content: "",
            attributes: HashMap::new(),
            childs: Vec::new(),
        }
    }

    /// Sets the name of the [`NodeBuilder`].
    #[must_use]
    pub const fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }

    /// Sets the content of the [`NodeBuilder`].
    #[must_use]
    pub const fn content(mut self, data: &'a str) -> Self {
        self.content = data;
        self
    }

    /// Adds all key-value pairs from an iterator to the attributes of the [`NodeBuilder`].
    #[must_use]
    pub fn attributes(mut self, attributes: impl IntoIterator<Item = (String, String)>) -> Self {
        self.attributes.extend(attributes);
        self
    }

    /// Adds an key-value pair as attributes to the [`NodeBuilder`].
    #[must_use]
    pub fn attribute(
        mut self,
        key: &(impl ToString + ?Sized),
        value: &(impl ToString + ?Sized),
    ) -> Self {
        self.attributes.insert(key.to_string(), value.to_string());
        self
    }

    /// Adds all nodes from an iterator to the [`NodeBuilder`] as childs.
    #[must_use]
    pub fn childs(mut self, childs: impl IntoIterator<Item = Node>) -> Self {
        self.childs.extend(childs);
        self
    }

    /// Adds a [`Node`] as a child to the [`Node`]
    #[must_use]
    pub fn child(mut self, child: impl Into<Node>) -> Self {
        self.childs.push(child.into());
        self
    }

    /// Adds a possible [`Node`] as a child to the [`NodeBuilder`].
    #[must_use]
    pub fn option_child(mut self, child: Option<impl Into<Node>>) -> Self {
        if let Some(child) = child {
            self.childs.push(child.into());
        }
        self
    }

    /// Tries to add the given struct as a child to the [`NodeBuilder`].
    ///
    /// # Errors
    ///
    /// This function will return an error if the child parameter cannot be succesfully be parsed into a node.
    pub fn try_child(mut self, child: impl TryInto<Node, Error = Error>) -> Result<Self> {
        self.childs.push(child.try_into()?);
        Ok(self)
    }

    /// Builds the node.
    #[must_use]
    pub fn build(self) -> Node {
        Node {
            name: self.name.to_owned(),
            content: self.content.to_owned(),
            attributes: self.attributes,
            childs: self.childs,
        }
    }
}

impl<'a> From<NodeBuilder<'a>> for Node {
    fn from(builder: NodeBuilder<'a>) -> Self {
        builder.build()
    }
}

/// Helper trait to transform an struct into an Iterator of Key-Value pairs for the attributes of a [`Node`].
pub trait IntoAttributes {
    /// Gets an iterator of an ([`String`], [`String`]) tuple.
    fn to_attributes(&self) -> impl IntoIterator<Item = (String, String)>;
}

/// Helper trait to transform an struct into an Iterator of nodes for the childs of a [`Node`].
pub trait IntoChilds {
    /// Gets an iterator of [Nodes](crate::node::Node).
    fn to_childs(&self) -> impl IntoIterator<Item = crate::node::Node>;
}

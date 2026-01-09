//! DOM node types and arena-based tree structure.

use compact_str::CompactString;
use smallvec::SmallVec;
use std::num::NonZeroU32;

/// A handle to a node in the DOM tree.
///
/// This is a lightweight index that can be used to access nodes in the [`Dom`] arena.
/// It is `Copy` and can be freely passed around without ownership concerns.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NodeId(NonZeroU32);

impl NodeId {
    /// Create a new NodeId from a raw index.
    ///
    /// # Safety
    /// The index must be valid (greater than 0).
    #[inline]
    pub(crate) fn new(index: u32) -> Self {
        NodeId(NonZeroU32::new(index).expect("NodeId index must be non-zero"))
    }

    /// Get the raw index value.
    #[inline]
    pub(crate) fn index(self) -> usize {
        (self.0.get() - 1) as usize
    }

    /// Create a NodeId from a 0-based array index.
    #[inline]
    pub(crate) fn from_index(index: usize) -> Self {
        NodeId::new((index + 1) as u32)
    }
}

/// The XML namespace of an element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Namespace {
    #[default]
    Html,
    Svg,
    MathML,
}

impl Namespace {
    /// Get the namespace URI string.
    pub fn uri(&self) -> &'static str {
        match self {
            Namespace::Html => "http://www.w3.org/1999/xhtml",
            Namespace::Svg => "http://www.w3.org/2000/svg",
            Namespace::MathML => "http://www.w3.org/1998/Math/MathML",
        }
    }

    /// Parse a namespace from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "html" | "http://www.w3.org/1999/xhtml" => Some(Namespace::Html),
            "svg" | "http://www.w3.org/2000/svg" => Some(Namespace::Svg),
            "mathml" | "math" | "http://www.w3.org/1998/Math/MathML" => Some(Namespace::MathML),
            _ => None,
        }
    }
}

/// DOCTYPE information.
#[derive(Debug, Clone, Default)]
pub struct Doctype {
    pub name: Option<CompactString>,
    pub public_id: Option<CompactString>,
    pub system_id: Option<CompactString>,
    pub force_quirks: bool,
}

/// An element with its tag name and attributes.
#[derive(Debug, Clone)]
pub struct Element {
    pub name: CompactString,
    pub namespace: Namespace,
    pub attrs: Attributes,
    /// For `<template>` elements, the content document fragment.
    pub template_content: Option<NodeId>,
}

impl Element {
    /// Create a new element with the given name and namespace.
    pub fn new(name: impl Into<CompactString>, namespace: Namespace) -> Self {
        Self {
            name: name.into(),
            namespace,
            attrs: Attributes::default(),
            template_content: None,
        }
    }

    /// Create a new HTML element.
    pub fn html(name: impl Into<CompactString>) -> Self {
        Self::new(name, Namespace::Html)
    }
}

/// Optimized attribute storage.
///
/// Uses a `SmallVec` to store up to 4 attributes inline without heap allocation.
/// Most elements have 0-4 attributes, so this avoids allocation in the common case.
#[derive(Debug, Clone, Default)]
pub struct Attributes {
    /// Stored as (name, value) pairs. Value is None for boolean attributes.
    inner: SmallVec<[(CompactString, Option<CompactString>); 4]>,
}

impl Attributes {
    /// Create an empty attribute set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get an attribute value by name.
    pub fn get(&self, name: &str) -> Option<Option<&str>> {
        self.inner
            .iter()
            .find(|(n, _)| n.as_str() == name)
            .map(|(_, v)| v.as_deref())
    }

    /// Check if an attribute exists.
    pub fn contains(&self, name: &str) -> bool {
        self.inner.iter().any(|(n, _)| n.as_str() == name)
    }

    /// Set an attribute value. Returns true if it was a new attribute.
    pub fn set(&mut self, name: impl Into<CompactString>, value: Option<CompactString>) -> bool {
        let name = name.into();
        for (n, v) in &mut self.inner {
            if n.as_str() == name.as_str() {
                *v = value;
                return false;
            }
        }
        self.inner.push((name, value));
        true
    }

    /// Remove an attribute. Returns the old value if it existed.
    pub fn remove(&mut self, name: &str) -> Option<Option<CompactString>> {
        if let Some(pos) = self.inner.iter().position(|(n, _)| n.as_str() == name) {
            Some(self.inner.remove(pos).1)
        } else {
            None
        }
    }

    /// Get the number of attributes.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if there are no attributes.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Iterate over all attributes.
    pub fn iter(&self) -> impl Iterator<Item = (&str, Option<&str>)> {
        self.inner
            .iter()
            .map(|(n, v)| (n.as_str(), v.as_deref()))
    }
}

/// Source location information for a node.
#[derive(Debug, Clone, Copy, Default)]
pub struct SourceLocation {
    /// 0-indexed byte offset in the source.
    pub offset: u32,
    /// 1-indexed line number.
    pub line: u32,
    /// 1-indexed column number.
    pub column: u32,
}

/// The kind of a DOM node.
#[derive(Debug, Clone)]
pub enum NodeKind {
    /// The document root.
    Document,
    /// A document fragment (e.g., template content).
    DocumentFragment,
    /// A DOCTYPE declaration.
    Doctype(Doctype),
    /// An element node (e.g., `<div>`, `<p>`).
    Element(Element),
    /// A text node.
    Text(CompactString),
    /// A comment node.
    Comment(CompactString),
}

impl NodeKind {
    /// Get the node name (tag name for elements, "#text" for text, etc.)
    pub fn name(&self) -> &str {
        match self {
            NodeKind::Document => "#document",
            NodeKind::DocumentFragment => "#document-fragment",
            NodeKind::Doctype(_) => "!doctype",
            NodeKind::Element(el) => &el.name,
            NodeKind::Text(_) => "#text",
            NodeKind::Comment(_) => "#comment",
        }
    }

    /// Check if this is an element node.
    pub fn is_element(&self) -> bool {
        matches!(self, NodeKind::Element(_))
    }

    /// Check if this is a text node.
    pub fn is_text(&self) -> bool {
        matches!(self, NodeKind::Text(_))
    }

    /// Get the element data if this is an element.
    pub fn as_element(&self) -> Option<&Element> {
        match self {
            NodeKind::Element(el) => Some(el),
            _ => None,
        }
    }

    /// Get mutable element data if this is an element.
    pub fn as_element_mut(&mut self) -> Option<&mut Element> {
        match self {
            NodeKind::Element(el) => Some(el),
            _ => None,
        }
    }

    /// Get the text content if this is a text node.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            NodeKind::Text(s) => Some(s),
            _ => None,
        }
    }

    /// Get the comment content if this is a comment node.
    pub fn as_comment(&self) -> Option<&str> {
        match self {
            NodeKind::Comment(s) => Some(s),
            _ => None,
        }
    }
}

/// Storage for a single node in the DOM arena.
#[derive(Debug, Clone)]
pub struct NodeData {
    /// The kind of node and its data.
    pub kind: NodeKind,
    /// Parent node, if any.
    pub parent: Option<NodeId>,
    /// First child node, if any.
    pub first_child: Option<NodeId>,
    /// Last child node, if any.
    pub last_child: Option<NodeId>,
    /// Previous sibling node, if any.
    pub prev_sibling: Option<NodeId>,
    /// Next sibling node, if any.
    pub next_sibling: Option<NodeId>,
    /// Source location, if tracked.
    pub origin: Option<SourceLocation>,
}

impl NodeData {
    /// Create a new node with the given kind.
    pub fn new(kind: NodeKind) -> Self {
        Self {
            kind,
            parent: None,
            first_child: None,
            last_child: None,
            prev_sibling: None,
            next_sibling: None,
            origin: None,
        }
    }

    /// Check if this node can have children.
    pub fn can_have_children(&self) -> bool {
        matches!(
            self.kind,
            NodeKind::Document | NodeKind::DocumentFragment | NodeKind::Element(_)
        )
    }
}

/// Arena-allocated DOM tree.
///
/// All nodes are stored in a contiguous vector, and relationships are
/// represented using [`NodeId`] indices. This design:
///
/// - Avoids reference counting overhead (no `Rc`/`Arc`)
/// - Provides cache-friendly memory layout
/// - Simplifies ownership (the `Dom` owns all nodes)
/// - Makes cloning straightforward
#[derive(Debug, Clone)]
pub struct Dom {
    nodes: Vec<NodeData>,
}

impl Default for Dom {
    fn default() -> Self {
        Self::new()
    }
}

impl Dom {
    /// Create a new empty DOM.
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Create a new DOM with a document root.
    pub fn with_document() -> (Self, NodeId) {
        let mut dom = Self::new();
        let root = dom.create_node(NodeKind::Document);
        (dom, root)
    }

    /// Create a new node and return its ID.
    pub fn create_node(&mut self, kind: NodeKind) -> NodeId {
        let id = NodeId::from_index(self.nodes.len());
        self.nodes.push(NodeData::new(kind));
        id
    }

    /// Create a new element node.
    pub fn create_element(&mut self, name: impl Into<CompactString>, ns: Namespace) -> NodeId {
        self.create_node(NodeKind::Element(Element::new(name, ns)))
    }

    /// Create a new text node.
    pub fn create_text(&mut self, data: impl Into<CompactString>) -> NodeId {
        self.create_node(NodeKind::Text(data.into()))
    }

    /// Create a new comment node.
    pub fn create_comment(&mut self, data: impl Into<CompactString>) -> NodeId {
        self.create_node(NodeKind::Comment(data.into()))
    }

    /// Create a new document fragment.
    pub fn create_document_fragment(&mut self) -> NodeId {
        self.create_node(NodeKind::DocumentFragment)
    }

    /// Get a reference to a node's data.
    pub fn get(&self, id: NodeId) -> &NodeData {
        &self.nodes[id.index()]
    }

    /// Get a mutable reference to a node's data.
    pub fn get_mut(&mut self, id: NodeId) -> &mut NodeData {
        &mut self.nodes[id.index()]
    }

    /// Append a child node to a parent.
    pub fn append_child(&mut self, parent: NodeId, child: NodeId) {
        // Remove from old parent if any
        if let Some(old_parent) = self.get(child).parent {
            self.remove_child(old_parent, child);
        }

        let parent_data = self.get(parent);
        let old_last = parent_data.last_child;

        // Update the new child's links
        {
            let child_data = self.get_mut(child);
            child_data.parent = Some(parent);
            child_data.prev_sibling = old_last;
            child_data.next_sibling = None;
        }

        // Update the old last child's next_sibling
        if let Some(old_last_id) = old_last {
            self.get_mut(old_last_id).next_sibling = Some(child);
        }

        // Update parent's child pointers
        let parent_data = self.get_mut(parent);
        if parent_data.first_child.is_none() {
            parent_data.first_child = Some(child);
        }
        parent_data.last_child = Some(child);
    }

    /// Insert a child before a reference node.
    pub fn insert_before(&mut self, parent: NodeId, child: NodeId, reference: Option<NodeId>) {
        if reference.is_none() {
            self.append_child(parent, child);
            return;
        }

        let reference = reference.unwrap();

        // Remove from old parent if any
        if let Some(old_parent) = self.get(child).parent {
            self.remove_child(old_parent, child);
        }

        let ref_prev = self.get(reference).prev_sibling;

        // Update child links
        {
            let child_data = self.get_mut(child);
            child_data.parent = Some(parent);
            child_data.prev_sibling = ref_prev;
            child_data.next_sibling = Some(reference);
        }

        // Update reference's prev_sibling
        self.get_mut(reference).prev_sibling = Some(child);

        // Update previous sibling's next_sibling, or parent's first_child
        if let Some(prev_id) = ref_prev {
            self.get_mut(prev_id).next_sibling = Some(child);
        } else {
            self.get_mut(parent).first_child = Some(child);
        }
    }

    /// Remove a child from its parent.
    pub fn remove_child(&mut self, parent: NodeId, child: NodeId) {
        let child_data = self.get(child);
        let prev = child_data.prev_sibling;
        let next = child_data.next_sibling;

        // Update sibling links
        if let Some(prev_id) = prev {
            self.get_mut(prev_id).next_sibling = next;
        } else {
            self.get_mut(parent).first_child = next;
        }

        if let Some(next_id) = next {
            self.get_mut(next_id).prev_sibling = prev;
        } else {
            self.get_mut(parent).last_child = prev;
        }

        // Clear child's links
        let child_data = self.get_mut(child);
        child_data.parent = None;
        child_data.prev_sibling = None;
        child_data.next_sibling = None;
    }

    /// Get an iterator over a node's children.
    pub fn children(&self, id: NodeId) -> ChildrenIter<'_> {
        ChildrenIter {
            dom: self,
            next: self.get(id).first_child,
        }
    }

    /// Get an iterator over a node's descendants in tree order.
    pub fn descendants(&self, id: NodeId) -> DescendantsIter<'_> {
        DescendantsIter {
            dom: self,
            root: id,
            current: Some(id),
        }
    }

    /// Get an iterator over a node's ancestors.
    pub fn ancestors(&self, id: NodeId) -> AncestorsIter<'_> {
        AncestorsIter {
            dom: self,
            next: self.get(id).parent,
        }
    }

    /// Get the number of nodes in the DOM.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the DOM is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

/// Iterator over a node's children.
pub struct ChildrenIter<'a> {
    dom: &'a Dom,
    next: Option<NodeId>,
}

impl<'a> Iterator for ChildrenIter<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.next?;
        self.next = self.dom.get(current).next_sibling;
        Some(current)
    }
}

/// Iterator over a node's descendants in tree order.
pub struct DescendantsIter<'a> {
    dom: &'a Dom,
    root: NodeId,
    current: Option<NodeId>,
}

impl<'a> Iterator for DescendantsIter<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;
        let node = self.dom.get(current);

        // Try to go to first child
        if let Some(child) = node.first_child {
            self.current = Some(child);
            return Some(current);
        }

        // Try to go to next sibling
        if current != self.root {
            if let Some(sibling) = node.next_sibling {
                self.current = Some(sibling);
                return Some(current);
            }

            // Go up and find an ancestor with a next sibling
            let mut ancestor = node.parent;
            while let Some(anc_id) = ancestor {
                if anc_id == self.root {
                    self.current = None;
                    return Some(current);
                }
                let anc = self.dom.get(anc_id);
                if let Some(sibling) = anc.next_sibling {
                    self.current = Some(sibling);
                    return Some(current);
                }
                ancestor = anc.parent;
            }
        }

        self.current = None;
        Some(current)
    }
}

/// Iterator over a node's ancestors.
pub struct AncestorsIter<'a> {
    dom: &'a Dom,
    next: Option<NodeId>,
}

impl<'a> Iterator for AncestorsIter<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.next?;
        self.next = self.dom.get(current).parent;
        Some(current)
    }
}

/// A convenience wrapper for accessing a node within a DOM.
#[derive(Clone, Copy)]
pub struct Node<'a> {
    dom: &'a Dom,
    id: NodeId,
}

impl<'a> Node<'a> {
    /// Create a new node wrapper.
    pub fn new(dom: &'a Dom, id: NodeId) -> Self {
        Self { dom, id }
    }

    /// Get the node ID.
    pub fn id(&self) -> NodeId {
        self.id
    }

    /// Get the underlying DOM.
    pub fn dom(&self) -> &'a Dom {
        self.dom
    }

    /// Get the node data.
    pub fn data(&self) -> &NodeData {
        self.dom.get(self.id)
    }

    /// Get the node kind.
    pub fn kind(&self) -> &NodeKind {
        &self.data().kind
    }

    /// Get the node name.
    pub fn name(&self) -> &str {
        self.kind().name()
    }

    /// Get the parent node, if any.
    pub fn parent(&self) -> Option<Node<'a>> {
        self.data().parent.map(|id| Node::new(self.dom, id))
    }

    /// Get an iterator over child nodes.
    pub fn children(&self) -> impl Iterator<Item = Node<'a>> {
        let dom = self.dom;
        self.dom.children(self.id).map(move |id| Node::new(dom, id))
    }

    /// Get the first child, if any.
    pub fn first_child(&self) -> Option<Node<'a>> {
        self.data().first_child.map(|id| Node::new(self.dom, id))
    }

    /// Get the last child, if any.
    pub fn last_child(&self) -> Option<Node<'a>> {
        self.data().last_child.map(|id| Node::new(self.dom, id))
    }

    /// Get the next sibling, if any.
    pub fn next_sibling(&self) -> Option<Node<'a>> {
        self.data().next_sibling.map(|id| Node::new(self.dom, id))
    }

    /// Get the previous sibling, if any.
    pub fn prev_sibling(&self) -> Option<Node<'a>> {
        self.data().prev_sibling.map(|id| Node::new(self.dom, id))
    }

    /// Check if this is an element node.
    pub fn is_element(&self) -> bool {
        self.kind().is_element()
    }

    /// Check if this is a text node.
    pub fn is_text(&self) -> bool {
        self.kind().is_text()
    }

    /// Get the element data if this is an element.
    pub fn as_element(&self) -> Option<&Element> {
        self.kind().as_element()
    }

    /// Get an attribute value.
    pub fn attr(&self, name: &str) -> Option<Option<&str>> {
        self.as_element().and_then(|el| el.attrs.get(name))
    }

    /// Check if the node has an attribute.
    pub fn has_attr(&self, name: &str) -> bool {
        self.as_element()
            .map(|el| el.attrs.contains(name))
            .unwrap_or(false)
    }

    /// Get the text content if this is a text node.
    pub fn text(&self) -> Option<&str> {
        self.kind().as_text()
    }
}

impl<'a> std::fmt::Debug for Node<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("id", &self.id)
            .field("kind", self.kind())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_nodes() {
        let mut dom = Dom::new();
        let doc = dom.create_node(NodeKind::Document);
        let div = dom.create_element("div", Namespace::Html);
        let text = dom.create_text("Hello");

        assert_eq!(dom.len(), 3);
        assert_eq!(dom.get(doc).kind.name(), "#document");
        assert_eq!(dom.get(div).kind.name(), "div");
        assert_eq!(dom.get(text).kind.as_text(), Some("Hello"));
    }

    #[test]
    fn test_append_child() {
        let mut dom = Dom::new();
        let doc = dom.create_node(NodeKind::Document);
        let div = dom.create_element("div", Namespace::Html);
        let text = dom.create_text("Hello");

        dom.append_child(doc, div);
        dom.append_child(div, text);

        assert_eq!(dom.get(doc).first_child, Some(div));
        assert_eq!(dom.get(doc).last_child, Some(div));
        assert_eq!(dom.get(div).parent, Some(doc));
        assert_eq!(dom.get(div).first_child, Some(text));
        assert_eq!(dom.get(text).parent, Some(div));
    }

    #[test]
    fn test_children_iter() {
        let mut dom = Dom::new();
        let parent = dom.create_element("div", Namespace::Html);
        let c1 = dom.create_element("span", Namespace::Html);
        let c2 = dom.create_element("p", Namespace::Html);
        let c3 = dom.create_text("text");

        dom.append_child(parent, c1);
        dom.append_child(parent, c2);
        dom.append_child(parent, c3);

        let children: Vec<_> = dom.children(parent).collect();
        assert_eq!(children, vec![c1, c2, c3]);
    }

    #[test]
    fn test_attributes() {
        let mut attrs = Attributes::new();

        assert!(attrs.set("id", Some("main".into())));
        assert!(attrs.set("class", Some("container".into())));
        assert!(attrs.set("disabled", None)); // boolean attr

        assert_eq!(attrs.get("id"), Some(Some("main")));
        assert_eq!(attrs.get("class"), Some(Some("container")));
        assert_eq!(attrs.get("disabled"), Some(None));
        assert_eq!(attrs.get("missing"), None);

        assert!(!attrs.set("id", Some("updated".into()))); // update
        assert_eq!(attrs.get("id"), Some(Some("updated")));

        assert_eq!(attrs.len(), 3);
        assert!(attrs.contains("disabled"));
        assert!(!attrs.contains("unknown"));
    }
}

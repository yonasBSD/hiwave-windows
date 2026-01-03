//! # RustKit DOM
//!
//! DOM implementation for the RustKit browser engine.
//! Uses html5ever for HTML parsing and constructs a traversable DOM tree.
//!
//! ## Design Goals
//!
//! 1. **Spec-compliant parsing**: html5ever implements the HTML5 parsing algorithm
//! 2. **Efficient tree structure**: Arena-based allocation for cache-friendly traversal
//! 3. **Query support**: Element lookup by ID, class, tag name
//! 4. **Mutation support**: Node insertion, removal, attribute modification
//! 5. **Event dispatch**: DOM Events with capture/bubble phases

pub mod events;
pub mod forms;

pub use events::{
    AddEventListenerOptions, DomEvent, Event, EventDispatcher, EventId, EventListenerCallback,
    EventPhase, EventTarget, FocusEventData, InputEventData, KeyboardEventData, MouseEventData,
};
pub use forms::{
    CheckableState, FormDataEntry, FormDataValue, FormEnctype, FormMethod, FormState, InputType,
    SelectionDirection, SelectionRange, TextEditState,
};

use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use thiserror::Error;
use tracing::debug;

/// Errors that can occur in DOM operations.
#[derive(Error, Debug)]
pub enum DomError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Node not found")]
    NodeNotFound,

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

/// Unique identifier for a DOM node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

impl NodeId {
    /// Create a new NodeId.
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    /// Get the raw ID value.
    pub fn raw(&self) -> usize {
        self.0
    }
}

/// Type of DOM node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    Document,
    DocumentType {
        name: String,
        public_id: String,
        system_id: String,
    },
    Element {
        tag_name: String,
        namespace: String,
        attributes: HashMap<String, String>,
    },
    Text(String),
    Comment(String),
    ProcessingInstruction {
        target: String,
        data: String,
    },
}

/// A DOM node.
#[derive(Debug)]
pub struct Node {
    /// Unique ID for this node.
    pub id: NodeId,
    /// Node type and associated data.
    pub node_type: NodeType,
    /// Parent node (weak reference to avoid cycles).
    parent: RefCell<Option<Weak<Node>>>,
    /// Child nodes.
    children: RefCell<Vec<Rc<Node>>>,
    /// Previous sibling.
    prev_sibling: RefCell<Option<Weak<Node>>>,
    /// Next sibling.
    next_sibling: RefCell<Option<Weak<Node>>>,
    /// Event target mixin for event handling.
    pub event_target: EventTarget,
}

impl Node {
    /// Create a new node.
    pub fn new(id: NodeId, node_type: NodeType) -> Rc<Self> {
        Rc::new(Self {
            id,
            node_type,
            parent: RefCell::new(None),
            children: RefCell::new(Vec::new()),
            prev_sibling: RefCell::new(None),
            next_sibling: RefCell::new(None),
            event_target: EventTarget::new(),
        })
    }

    /// Get the tag name for element nodes.
    pub fn tag_name(&self) -> Option<&str> {
        match &self.node_type {
            NodeType::Element { tag_name, .. } => Some(tag_name),
            _ => None,
        }
    }

    /// Get an attribute value.
    pub fn get_attribute(&self, name: &str) -> Option<&str> {
        match &self.node_type {
            NodeType::Element { attributes, .. } => attributes.get(name).map(|s| s.as_str()),
            _ => None,
        }
    }

    /// Get the text content.
    pub fn text_content(&self) -> String {
        let mut result = String::new();
        self.collect_text(&mut result);
        result
    }

    fn collect_text(&self, result: &mut String) {
        match &self.node_type {
            NodeType::Text(text) => result.push_str(text),
            _ => {
                for child in self.children.borrow().iter() {
                    child.collect_text(result);
                }
            }
        }
    }

    /// Get parent node.
    pub fn parent(&self) -> Option<Rc<Node>> {
        self.parent.borrow().as_ref().and_then(|w| w.upgrade())
    }

    /// Get child nodes.
    pub fn children(&self) -> Vec<Rc<Node>> {
        self.children.borrow().clone()
    }

    /// Get first child.
    pub fn first_child(&self) -> Option<Rc<Node>> {
        self.children.borrow().first().cloned()
    }

    /// Get last child.
    pub fn last_child(&self) -> Option<Rc<Node>> {
        self.children.borrow().last().cloned()
    }

    /// Get previous sibling.
    pub fn previous_sibling(&self) -> Option<Rc<Node>> {
        self.prev_sibling
            .borrow()
            .as_ref()
            .and_then(|w| w.upgrade())
    }

    /// Get next sibling.
    pub fn next_sibling(&self) -> Option<Rc<Node>> {
        self.next_sibling
            .borrow()
            .as_ref()
            .and_then(|w| w.upgrade())
    }

    /// Check if this is an element node.
    pub fn is_element(&self) -> bool {
        matches!(self.node_type, NodeType::Element { .. })
    }

    /// Check if this is a text node.
    pub fn is_text(&self) -> bool {
        matches!(self.node_type, NodeType::Text(_))
    }

    /// Append a child node.
    pub fn append_child(self: &Rc<Self>, child: Rc<Node>) {
        // Update child's parent
        *child.parent.borrow_mut() = Some(Rc::downgrade(self));

        // Update sibling links
        if let Some(last) = self.last_child() {
            *last.next_sibling.borrow_mut() = Some(Rc::downgrade(&child));
            *child.prev_sibling.borrow_mut() = Some(Rc::downgrade(&last));
        }

        // Add to children
        self.children.borrow_mut().push(child);
    }
}

/// A complete DOM document.
pub struct Document {
    /// Root node of the document.
    root: Rc<Node>,
    /// All nodes indexed by ID.
    nodes: HashMap<NodeId, Rc<Node>>,
    /// Elements indexed by ID attribute.
    elements_by_id: HashMap<String, Rc<Node>>,
    /// Next node ID.
    next_id: Cell<usize>,
}

impl Document {
    /// Create a new empty document.
    pub fn new() -> Self {
        let root = Node::new(NodeId::new(0), NodeType::Document);
        let mut nodes = HashMap::new();
        nodes.insert(NodeId::new(0), root.clone());

        Self {
            root,
            nodes,
            elements_by_id: HashMap::new(),
            next_id: Cell::new(1),
        }
    }

    /// Parse HTML and create a document.
    pub fn parse_html(html: &str) -> Result<Self, DomError> {
        debug!(len = html.len(), "Parsing HTML");

        let dom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut html.as_bytes())
            .map_err(|e| DomError::ParseError(e.to_string()))?;

        let mut doc = Document::new();
        doc.convert_rcdom(&dom.document, &doc.root.clone());

        // Index elements by ID
        doc.index_elements();

        debug!(node_count = doc.nodes.len(), "HTML parsed");
        Ok(doc)
    }

    fn convert_rcdom(&mut self, handle: &Handle, parent: &Rc<Node>) {
        for child_handle in handle.children.borrow().iter() {
            let node_type = match &child_handle.data {
                NodeData::Document => continue, // Skip document node itself
                NodeData::Doctype {
                    name,
                    public_id,
                    system_id,
                } => NodeType::DocumentType {
                    name: name.to_string(),
                    public_id: public_id.to_string(),
                    system_id: system_id.to_string(),
                },
                NodeData::Element { name, attrs, .. } => {
                    let mut attributes = HashMap::new();
                    for attr in attrs.borrow().iter() {
                        attributes.insert(attr.name.local.to_string(), attr.value.to_string());
                    }
                    NodeType::Element {
                        tag_name: name.local.to_string(),
                        namespace: name.ns.to_string(),
                        attributes,
                    }
                }
                NodeData::Text { contents } => NodeType::Text(contents.borrow().to_string()),
                NodeData::Comment { contents } => NodeType::Comment(contents.to_string()),
                NodeData::ProcessingInstruction { target, contents } => {
                    NodeType::ProcessingInstruction {
                        target: target.to_string(),
                        data: contents.to_string(),
                    }
                }
            };

            let id = NodeId::new(self.next_id.get());
            self.next_id.set(self.next_id.get() + 1);

            let node = Node::new(id, node_type);
            self.nodes.insert(id, node.clone());
            parent.append_child(node.clone());

            // Recurse for children
            self.convert_rcdom(child_handle, &node);
        }
    }

    fn index_elements(&mut self) {
        for node in self.nodes.values() {
            if let Some(id) = node.get_attribute("id") {
                self.elements_by_id.insert(id.to_string(), node.clone());
            }
        }
    }

    /// Get the document root.
    pub fn root(&self) -> &Rc<Node> {
        &self.root
    }

    /// Get the document element (<html>).
    pub fn document_element(&self) -> Option<Rc<Node>> {
        self.root
            .children()
            .into_iter()
            .find(|n| n.tag_name() == Some("html"))
    }

    /// Get the <head> element.
    pub fn head(&self) -> Option<Rc<Node>> {
        self.document_element()?
            .children()
            .into_iter()
            .find(|n| n.tag_name() == Some("head"))
    }

    /// Get the <body> element.
    pub fn body(&self) -> Option<Rc<Node>> {
        self.document_element()?
            .children()
            .into_iter()
            .find(|n| n.tag_name() == Some("body"))
    }

    /// Get element by ID.
    pub fn get_element_by_id(&self, id: &str) -> Option<Rc<Node>> {
        self.elements_by_id.get(id).cloned()
    }

    /// Get elements by tag name.
    pub fn get_elements_by_tag_name(&self, tag_name: &str) -> Vec<Rc<Node>> {
        let tag_name_lower = tag_name.to_lowercase();
        self.nodes
            .values()
            .filter(|n| {
                n.tag_name()
                    .map(|t| t.to_lowercase() == tag_name_lower)
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    /// Get elements by class name.
    pub fn get_elements_by_class_name(&self, class_name: &str) -> Vec<Rc<Node>> {
        self.nodes
            .values()
            .filter(|n| {
                n.get_attribute("class")
                    .map(|c| c.split_whitespace().any(|cls| cls == class_name))
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    /// Get node by ID.
    pub fn get_node(&self, id: NodeId) -> Option<Rc<Node>> {
        self.nodes.get(&id).cloned()
    }

    /// Get the title of the document.
    pub fn title(&self) -> Option<String> {
        let head = self.head()?;
        let title_elem = head
            .children()
            .into_iter()
            .find(|n| n.tag_name() == Some("title"))?;
        Some(title_elem.text_content())
    }

    /// Traverse all nodes depth-first.
    pub fn traverse<F>(&self, mut callback: F)
    where
        F: FnMut(&Rc<Node>),
    {
        self.traverse_node(&self.root, &mut callback);
    }

    #[allow(clippy::only_used_in_recursion)]
    fn traverse_node<F>(&self, node: &Rc<Node>, callback: &mut F)
    where
        F: FnMut(&Rc<Node>),
    {
        callback(node);
        for child in node.children() {
            self.traverse_node(&child, callback);
        }
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

/// Query selector support (basic).
pub struct QuerySelector;

impl QuerySelector {
    /// Select elements matching a simple selector.
    /// Supports: tag, #id, .class
    pub fn select(doc: &Document, selector: &str) -> Vec<Rc<Node>> {
        let selector = selector.trim();

        if let Some(id) = selector.strip_prefix('#') {
            // ID selector
            doc.get_element_by_id(id).into_iter().collect()
        } else if let Some(class) = selector.strip_prefix('.') {
            // Class selector
            doc.get_elements_by_class_name(class)
        } else {
            // Tag selector
            doc.get_elements_by_tag_name(selector)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_html() {
        let html = r#"<!DOCTYPE html>
<html>
<head><title>Test</title></head>
<body><p id="main">Hello, world!</p></body>
</html>"#;

        let doc = Document::parse_html(html).unwrap();

        // Check structure
        assert!(doc.document_element().is_some());
        assert!(doc.head().is_some());
        assert!(doc.body().is_some());

        // Check title
        assert_eq!(doc.title(), Some("Test".to_string()));

        // Check ID lookup
        let main = doc.get_element_by_id("main").unwrap();
        assert_eq!(main.tag_name(), Some("p"));
        assert_eq!(main.text_content(), "Hello, world!");
    }

    #[test]
    fn test_query_selector() {
        let html = r#"<html>
<body>
    <div class="container">
        <p id="first">First</p>
        <p class="highlight">Second</p>
        <span class="highlight">Third</span>
    </div>
</body>
</html>"#;

        let doc = Document::parse_html(html).unwrap();

        // ID selector
        let by_id = QuerySelector::select(&doc, "#first");
        assert_eq!(by_id.len(), 1);
        assert_eq!(by_id[0].text_content(), "First");

        // Class selector
        let by_class = QuerySelector::select(&doc, ".highlight");
        assert_eq!(by_class.len(), 2);

        // Tag selector
        let by_tag = QuerySelector::select(&doc, "p");
        assert_eq!(by_tag.len(), 2);
    }

    #[test]
    fn test_traversal() {
        let html = "<html><head></head><body><div><p>Text</p></div></body></html>";
        let doc = Document::parse_html(html).unwrap();

        let mut count = 0;
        doc.traverse(|_| count += 1);
        assert!(count > 0);
    }

    #[test]
    fn test_node_relationships() {
        let html = "<html><body><p>A</p><p>B</p><p>C</p></body></html>";
        let doc = Document::parse_html(html).unwrap();

        let body = doc.body().unwrap();
        let paragraphs: Vec<_> = body
            .children()
            .into_iter()
            .filter(|n| n.is_element())
            .collect();

        assert_eq!(paragraphs.len(), 3);

        // Check sibling relationships
        let first = &paragraphs[0];
        let second = &paragraphs[1];
        let third = &paragraphs[2];

        assert!(
            first.previous_sibling().is_none() || !first.previous_sibling().unwrap().is_element()
        );
        assert_eq!(
            second
                .previous_sibling()
                .map(|n| n.text_content().trim().to_string()),
            Some("A".to_string())
        );
    }
}

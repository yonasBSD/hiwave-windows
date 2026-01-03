//! # RustKit HTML
//!
//! HTML5 parser for the RustKit browser engine.
//!
//! This crate provides a tokenizer and tree builder that work together
//! to parse HTML into a DOM tree via a sink interface.

pub mod entities;
pub mod tokenizer;
pub mod tree_builder;

use thiserror::Error;

/// Errors that can occur during HTML parsing.
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Tokenizer error: {0}")]
    TokenizerError(String),

    #[error("Tree builder error: {0}")]
    TreeBuilderError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for HTML parsing operations.
pub type ParseResult<T> = Result<T, ParseError>;

/// Trait for receiving parse events and building a tree structure.
///
/// This trait is implemented by DOM builders that want to construct
/// a tree from HTML parsing events.
pub trait TreeSink {
    /// The type used to identify nodes in the tree.
    type NodeId: Clone;

    /// Called when a doctype declaration is encountered.
    fn doctype(&mut self, name: String, public_id: String, system_id: String);

    /// Called when a start tag is encountered.
    /// Returns the node ID for the newly created element.
    fn start_element(
        &mut self,
        name: String,
        attrs: Vec<(String, String)>,
        self_closing: bool,
    ) -> Self::NodeId;

    /// Called when an end tag is encountered.
    fn end_element(&mut self, name: String);

    /// Called when text content is encountered.
    fn text(&mut self, data: String);

    /// Called when a comment is encountered.
    fn comment(&mut self, data: String);

    /// Called to get the current node being processed.
    fn current_node(&self) -> Option<Self::NodeId>;

    /// Called to check if the parser is in a specific context.
    fn in_scope(&self, tag_name: &str) -> bool;

    /// Called to pop elements until a specific tag is found.
    fn pop_until(&mut self, tag_name: &str);

    /// Called to close a p element if one is in button scope.
    fn close_p_element_in_button_scope(&mut self);

    /// Called to reconstruct active formatting elements.
    fn reconstruct_active_formatting_elements(&mut self);
}

/// Parse HTML from a string using the provided sink.
pub fn parse<S: TreeSink>(html: &str, mut sink: S) -> ParseResult<S> {
    let tokens = tokenizer::tokenize(html)?;
    sink = tree_builder::build_tree(tokens, sink)?;
    Ok(sink)
}

/// Parse HTML from bytes using the provided sink.
pub fn parse_bytes<S: TreeSink>(html: &[u8], sink: S) -> ParseResult<S> {
    let html_str = std::str::from_utf8(html)
        .map_err(|e| ParseError::TokenizerError(format!("Invalid UTF-8: {}", e)))?;
    parse(html_str, sink)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestNode {
        name: String,
        attrs: Vec<(String, String)>,
        children: Vec<TestNode>,
    }

    struct TestSink {
        nodes: Vec<TestNode>,
        stack: Vec<usize>,
    }

    impl TestSink {
        fn new() -> Self {
            Self {
                nodes: vec![TestNode {
                    name: "root".to_string(),
                    attrs: vec![],
                    children: vec![],
                }],
                stack: vec![0],
            }
        }
    }

    impl TreeSink for TestSink {
        type NodeId = usize;

        fn doctype(&mut self, _name: String, _public_id: String, _system_id: String) {
            // Test sink ignores doctype
        }

        fn start_element(
            &mut self,
            name: String,
            attrs: Vec<(String, String)>,
            _self_closing: bool,
        ) -> Self::NodeId {
            let node = TestNode {
                name,
                attrs,
                children: vec![],
            };
            let node_id = self.nodes.len();
            self.nodes.push(node);
            
            if let Some(&parent_id) = self.stack.last() {
                // In a real implementation, we'd add to parent's children
                // For this test, we just track the structure
            }
            
            self.stack.push(node_id);
            node_id
        }

        fn end_element(&mut self, _name: String) {
            self.stack.pop();
        }

        fn text(&mut self, _data: String) {
            // Test sink ignores text for now
        }

        fn comment(&mut self, _data: String) {
            // Test sink ignores comments
        }

        fn current_node(&self) -> Option<Self::NodeId> {
            self.stack.last().copied()
        }

        fn in_scope(&self, _tag_name: &str) -> bool {
            false
        }

        fn pop_until(&mut self, _tag_name: &str) {
            // Simplified for tests
        }

        fn close_p_element_in_button_scope(&mut self) {
            // Simplified for tests
        }

        fn reconstruct_active_formatting_elements(&mut self) {
            // Simplified for tests
        }
    }

    #[test]
    fn test_basic_parse() {
        let html = "<html><body><p>Hello</p></body></html>";
        let sink = TestSink::new();
        parse(html, sink).unwrap();
        // Basic smoke test - more detailed tests in submodules
    }

    #[test]
    fn test_empty_html() {
        let html = "";
        let sink = TestSink::new();
        parse(html, sink).unwrap();
    }

    #[test]
    fn test_parse_bytes() {
        let html = b"<html></html>";
        let sink = TestSink::new();
        parse_bytes(html, sink).unwrap();
    }
}


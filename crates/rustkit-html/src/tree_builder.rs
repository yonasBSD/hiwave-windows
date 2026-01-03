//! HTML tree builder.
//!
//! Implements a simplified HTML5 tree construction algorithm with
//! insertion modes and error recovery.

use crate::tokenizer::Token;
use crate::{ParseResult, TreeSink};
use tracing::trace;

/// Insertion mode for tree construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InsertionMode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    AfterHead,
    InBody,
    AfterBody,
    AfterAfterBody,
}

/// Void elements that cannot have children.
const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param",
    "source", "track", "wbr",
];

/// Elements that cause implicit closure of p elements.
const P_CLOSING_ELEMENTS: &[&str] = &[
    "address", "article", "aside", "blockquote", "div", "dl", "fieldset", "figcaption",
    "figure", "footer", "form", "h1", "h2", "h3", "h4", "h5", "h6", "header", "hgroup", "hr",
    "main", "nav", "ol", "p", "pre", "section", "table", "ul",
];

/// HTML tree builder.
pub struct TreeBuilder<S: TreeSink> {
    sink: S,
    mode: InsertionMode,
    open_elements: Vec<(String, S::NodeId)>,
    foster_parenting: bool,
    scripting: bool,
    /// Buffer for accumulating consecutive text characters
    text_buffer: String,
}

impl<S: TreeSink> TreeBuilder<S> {
    pub fn new(sink: S) -> Self {
        Self {
            sink,
            mode: InsertionMode::Initial,
            open_elements: Vec::new(),
            foster_parenting: false,
            scripting: false,
            text_buffer: String::new(),
        }
    }
    
    fn flush_text(&mut self) {
        if !self.text_buffer.is_empty() {
            let text = std::mem::take(&mut self.text_buffer);
            self.sink.text(text);
        }
    }

    fn current_node(&self) -> Option<&(String, S::NodeId)> {
        self.open_elements.last()
    }

    fn current_node_name(&self) -> Option<&str> {
        self.current_node().map(|(name, _)| name.as_str())
    }

    fn has_element_in_scope(&self, tag_name: &str) -> bool {
        for (name, _) in self.open_elements.iter().rev() {
            if name == tag_name {
                return true;
            }
            // Scope-limiting elements
            if matches!(
                name.as_str(),
                "applet" | "caption" | "html" | "table" | "td" | "th" | "marquee" | "object" | "template"
            ) {
                return false;
            }
        }
        false
    }

    fn pop_until(&mut self, tag_name: &str) {
        while let Some((name, _)) = self.open_elements.pop() {
            if name == tag_name {
                break;
            }
        }
    }

    fn close_p_element(&mut self) {
        if self.has_element_in_scope("p") {
            self.sink.close_p_element_in_button_scope();
            self.pop_until("p");
        }
    }

    pub fn build(mut self, tokens: Vec<Token>) -> ParseResult<S> {
        for token in tokens {
            self.process_token(token)?;
        }
        Ok(self.sink)
    }

    fn process_token(&mut self, token: Token) -> ParseResult<()> {
        trace!(mode = ?self.mode, token = ?token, "Processing token");

        match self.mode {
            InsertionMode::Initial => self.handle_initial(token)?,
            InsertionMode::BeforeHtml => self.handle_before_html(token)?,
            InsertionMode::BeforeHead => self.handle_before_head(token)?,
            InsertionMode::InHead => self.handle_in_head(token)?,
            InsertionMode::AfterHead => self.handle_after_head(token)?,
            InsertionMode::InBody => self.handle_in_body(token)?,
            InsertionMode::AfterBody => self.handle_after_body(token)?,
            InsertionMode::AfterAfterBody => self.handle_after_after_body(token)?,
        }

        Ok(())
    }

    fn handle_initial(&mut self, token: Token) -> ParseResult<()> {
        match token {
            Token::Doctype {
                name,
                public_id,
                system_id,
            } => {
                self.sink.doctype(name, public_id, system_id);
                self.mode = InsertionMode::BeforeHtml;
            }
            Token::Character(ch) if ch.is_whitespace() => {
                // Ignore whitespace
            }
            Token::Comment(data) => {
                self.sink.comment(data);
            }
            _ => {
                // No doctype, switch to quirks mode
                self.mode = InsertionMode::BeforeHtml;
                self.process_token(token)?;
            }
        }
        Ok(())
    }

    fn handle_before_html(&mut self, token: Token) -> ParseResult<()> {
        match token {
            Token::StartTag {
                name, attrs, self_closing: _
            } if name == "html" => {
                let node_id = self.sink.start_element(name.clone(), attrs.into_iter().collect(), false);
                self.open_elements.push((name, node_id));
                self.mode = InsertionMode::BeforeHead;
            }
            Token::Character(ch) if ch.is_whitespace() => {
                // Ignore
            }
            Token::Comment(data) => {
                self.sink.comment(data);
            }
            _ => {
                // Implied html start tag
                let node_id = self.sink.start_element("html".to_string(), vec![], false);
                self.open_elements.push(("html".to_string(), node_id));
                self.mode = InsertionMode::BeforeHead;
                self.process_token(token)?;
            }
        }
        Ok(())
    }

    fn handle_before_head(&mut self, token: Token) -> ParseResult<()> {
        match token {
            Token::StartTag {
                name, attrs, self_closing: _
            } if name == "head" => {
                let node_id = self.sink.start_element(name.clone(), attrs.into_iter().collect(), false);
                self.open_elements.push((name, node_id));
                self.mode = InsertionMode::InHead;
            }
            Token::Character(ch) if ch.is_whitespace() => {
                // Ignore
            }
            Token::Comment(data) => {
                self.sink.comment(data);
            }
            _ => {
                // Implied head start tag
                let node_id = self.sink.start_element("head".to_string(), vec![], false);
                self.open_elements.push(("head".to_string(), node_id));
                self.mode = InsertionMode::InHead;
                self.process_token(token)?;
            }
        }
        Ok(())
    }

    fn handle_in_head(&mut self, token: Token) -> ParseResult<()> {
        match token {
            Token::Character(ch) => {
                // Add all characters to the text buffer (whitespace and non-whitespace)
                self.text_buffer.push(ch);
            }
            Token::Comment(data) => {
                self.flush_text();
                self.sink.comment(data);
            }
            Token::StartTag {
                name, attrs, self_closing: _
            } if matches!(name.as_str(), "base" | "basefont" | "bgsound" | "link" | "meta") => {
                let _node_id = self.sink.start_element(name.clone(), attrs.into_iter().collect(), true);
                // Void element, no end tag needed
            }
            Token::StartTag {
                name, attrs, self_closing: _
            } if matches!(name.as_str(), "title" | "style" | "script") => {
                let node_id = self.sink.start_element(name.clone(), attrs.into_iter().collect(), false);
                self.open_elements.push((name, node_id));
                // Note: In a full implementation, we'd switch to RCDATA/RAWTEXT mode here
            }
            Token::EndTag { name } if matches!(name.as_str(), "title" | "style" | "script") => {
                self.flush_text();
                if let Some((tag_name, _)) = self.open_elements.pop() {
                    self.sink.end_element(tag_name);
                }
            }
            Token::EndTag { name } if name == "head" => {
                self.flush_text();
                if let Some((tag_name, _)) = self.open_elements.pop() {
                    self.sink.end_element(tag_name);
                }
                self.mode = InsertionMode::AfterHead;
            }
            Token::EndTag { name } if matches!(name.as_str(), "body" | "html" | "br") => {
                // Act as if </head> seen
                self.flush_text();
                if let Some((tag_name, _)) = self.open_elements.pop() {
                    self.sink.end_element(tag_name);
                }
                self.mode = InsertionMode::AfterHead;
                self.process_token(Token::EndTag { name })?;
            }
            Token::StartTag { .. } => {
                // Any other start tag - close head implicitly
                self.flush_text();
                if let Some((tag_name, _)) = self.open_elements.pop() {
                    self.sink.end_element(tag_name);
                }
                self.mode = InsertionMode::AfterHead;
                self.process_token(token)?;
            }
            _ => {
                // Ignore or handle as error
            }
        }
        Ok(())
    }

    fn handle_after_head(&mut self, token: Token) -> ParseResult<()> {
        match token {
            Token::Character(ch) if ch.is_whitespace() => {
                self.text_buffer.push(ch);
            }
            Token::Comment(data) => {
                self.sink.comment(data);
            }
            Token::StartTag {
                name, attrs, self_closing: _
            } if name == "body" => {
                let node_id = self.sink.start_element(name.clone(), attrs.into_iter().collect(), false);
                self.open_elements.push((name, node_id));
                self.mode = InsertionMode::InBody;
            }
            _ => {
                // Implied body start tag
                let node_id = self.sink.start_element("body".to_string(), vec![], false);
                self.open_elements.push(("body".to_string(), node_id));
                self.mode = InsertionMode::InBody;
                self.process_token(token)?;
            }
        }
        Ok(())
    }

    fn handle_in_body(&mut self, token: Token) -> ParseResult<()> {
        match token {
            Token::Character(ch) => {
                if ch == '\0' {
                    // Ignore null characters
                    return Ok(());
                }
                
                // Accumulate text characters
                self.text_buffer.push(ch);
            }
            Token::Comment(data) => {
                self.flush_text();
                self.sink.comment(data);
            }
            Token::StartTag {
                name,
                attrs,
                self_closing,
            } => {
                // Flush any pending text before starting a new element
                self.flush_text();
                
                // Close p element if necessary
                if P_CLOSING_ELEMENTS.contains(&name.as_str()) {
                    self.close_p_element();
                }

                let is_void = VOID_ELEMENTS.contains(&name.as_str());
                let node_id = self.sink.start_element(
                    name.clone(),
                    attrs.into_iter().collect(),
                    self_closing || is_void,
                );

                if !is_void && !self_closing {
                    self.open_elements.push((name, node_id));
                }
            }
            Token::EndTag { name } => {
                self.flush_text();
                
                // Find matching open element
                let mut found = false;
                for i in (0..self.open_elements.len()).rev() {
                    if self.open_elements[i].0 == name {
                        // Close all elements up to and including this one
                        for _ in i..self.open_elements.len() {
                            let (tag_name, _) = self.open_elements.pop().unwrap();
                            self.sink.end_element(tag_name);
                        }
                        found = true;
                        break;
                    }
                }

                if !found {
                    // Parse error - ignore
                    trace!("Unmatched end tag: {}", name);
                }

                // Check if we closed body
                if name == "body" {
                    self.mode = InsertionMode::AfterBody;
                }
            }
            Token::Eof => {
                self.flush_text();
                
                // Close all open elements
                while let Some((tag_name, _)) = self.open_elements.pop() {
                    self.sink.end_element(tag_name);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_after_body(&mut self, token: Token) -> ParseResult<()> {
        match token {
            Token::Character(ch) if ch.is_whitespace() => {
                // Reprocess in "in body" mode
                self.mode = InsertionMode::InBody;
                self.process_token(Token::Character(ch))?;
                self.mode = InsertionMode::AfterBody;
            }
            Token::Comment(data) => {
                self.sink.comment(data);
            }
            Token::EndTag { name } if name == "html" => {
                self.mode = InsertionMode::AfterAfterBody;
            }
            Token::Eof => {
                // Done
            }
            _ => {
                // Parse error - reprocess in "in body" mode
                self.mode = InsertionMode::InBody;
                self.process_token(token)?;
            }
        }
        Ok(())
    }

    fn handle_after_after_body(&mut self, token: Token) -> ParseResult<()> {
        match token {
            Token::Comment(data) => {
                self.sink.comment(data);
            }
            Token::Character(ch) if ch.is_whitespace() => {
                self.mode = InsertionMode::InBody;
                self.process_token(Token::Character(ch))?;
                self.mode = InsertionMode::AfterAfterBody;
            }
            Token::Eof => {
                // Done
            }
            _ => {
                // Parse error - reprocess in "in body" mode
                self.mode = InsertionMode::InBody;
                self.process_token(token)?;
            }
        }
        Ok(())
    }
}

/// Build a tree from tokens using the provided sink.
pub fn build_tree<S: TreeSink>(tokens: Vec<Token>, sink: S) -> ParseResult<S> {
    let builder = TreeBuilder::new(sink);
    builder.build(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::tokenize;

    #[derive(Debug)]
    struct TestSink {
        events: Vec<String>,
    }

    impl TestSink {
        fn new() -> Self {
            Self { events: Vec::new() }
        }
    }

    impl TreeSink for TestSink {
        type NodeId = usize;

        fn doctype(&mut self, name: String, _public_id: String, _system_id: String) {
            self.events.push(format!("doctype:{}", name));
        }

        fn start_element(
            &mut self,
            name: String,
            attrs: Vec<(String, String)>,
            _self_closing: bool,
        ) -> Self::NodeId {
            let attr_str = attrs
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(" ");
            if attr_str.is_empty() {
                self.events.push(format!("start:{}", name));
            } else {
                self.events.push(format!("start:{}[{}]", name, attr_str));
            }
            self.events.len()
        }

        fn end_element(&mut self, name: String) {
            self.events.push(format!("end:{}", name));
        }

        fn text(&mut self, data: String) {
            if !data.trim().is_empty() {
                self.events.push(format!("text:{}", data));
            }
        }

        fn comment(&mut self, data: String) {
            self.events.push(format!("comment:{}", data));
        }

        fn current_node(&self) -> Option<Self::NodeId> {
            Some(0)
        }

        fn in_scope(&self, _tag_name: &str) -> bool {
            false
        }

        fn pop_until(&mut self, _tag_name: &str) {}

        fn close_p_element_in_button_scope(&mut self) {}

        fn reconstruct_active_formatting_elements(&mut self) {}
    }

    #[test]
    fn test_simple_document() {
        let html = "<html><head></head><body></body></html>";
        let tokens = tokenize(html).unwrap();
        let sink = TestSink::new();
        let result = build_tree(tokens, sink).unwrap();
        
        assert!(result.events.contains(&"start:html".to_string()));
        assert!(result.events.contains(&"start:head".to_string()));
        assert!(result.events.contains(&"start:body".to_string()));
    }

    #[test]
    fn test_implicit_tags() {
        let html = "<p>Hello</p>";
        let tokens = tokenize(html).unwrap();
        let sink = TestSink::new();
        let result = build_tree(tokens, sink).unwrap();
        
        // Should have implicit html, head, body
        assert!(result.events.contains(&"start:html".to_string()));
        assert!(result.events.contains(&"start:body".to_string()));
        assert!(result.events.contains(&"start:p".to_string()));
    }

    #[test]
    fn test_doctype() {
        let html = "<!DOCTYPE html><html></html>";
        let tokens = tokenize(html).unwrap();
        let sink = TestSink::new();
        let result = build_tree(tokens, sink).unwrap();
        
        assert!(result.events.contains(&"doctype:html".to_string()));
    }

    #[test]
    fn test_nested_elements() {
        let html = "<div><span>text</span></div>";
        let tokens = tokenize(html).unwrap();
        let sink = TestSink::new();
        let result = build_tree(tokens, sink).unwrap();
        
        assert!(result.events.contains(&"start:div".to_string()));
        assert!(result.events.contains(&"start:span".to_string()));
        assert!(result.events.contains(&"end:span".to_string()));
        assert!(result.events.contains(&"end:div".to_string()));
    }

    #[test]
    fn test_void_elements() {
        let html = "<p>Line 1<br>Line 2</p>";
        let tokens = tokenize(html).unwrap();
        let sink = TestSink::new();
        let result = build_tree(tokens, sink).unwrap();
        
        assert!(result.events.contains(&"start:br".to_string()));
        // br should not have an end tag
        assert!(!result.events.contains(&"end:br".to_string()));
    }

    #[test]
    fn test_malformed_nesting() {
        let html = "<div><span></div></span>";
        let tokens = tokenize(html).unwrap();
        let sink = TestSink::new();
        let result = build_tree(tokens, sink).unwrap();
        
        // Should recover gracefully
        assert!(!result.events.is_empty());
    }
}


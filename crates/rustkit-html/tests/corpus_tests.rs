//! Comprehensive corpus tests for HTML parser

use rustkit_html::{parse, TreeSink};
use std::collections::HashMap;

#[derive(Debug)]
struct TestSink {
    events: Vec<String>,
    node_count: usize,
}

impl TestSink {
    fn new() -> Self {
        Self {
            events: Vec::new(),
            node_count: 0,
        }
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
            .map(|(k, v)| if v.is_empty() { k.clone() } else { format!("{}={}", k, v) })
            .collect::<Vec<_>>()
            .join(" ");
        if attr_str.is_empty() {
            self.events.push(format!("start:{}", name));
        } else {
            self.events.push(format!("start:{}[{}]", name, attr_str));
        }
        self.node_count += 1;
        self.node_count
    }

    fn end_element(&mut self, name: String) {
        self.events.push(format!("end:{}", name));
    }

    fn text(&mut self, data: String) {
        if !data.trim().is_empty() {
            self.events.push(format!("text:{}", data.trim()));
        }
    }

    fn comment(&mut self, data: String) {
        self.events.push(format!("comment:{}", data));
    }

    fn current_node(&self) -> Option<Self::NodeId> {
        if self.node_count > 0 {
            Some(self.node_count)
        } else {
            None
        }
    }

    fn in_scope(&self, _tag_name: &str) -> bool {
        false
    }

    fn pop_until(&mut self, _tag_name: &str) {}

    fn close_p_element_in_button_scope(&mut self) {}

    fn reconstruct_active_formatting_elements(&mut self) {}
}

#[test]
fn test_simple_page() {
    let html = r#"<!DOCTYPE html>
<html>
<head><title>Test</title></head>
<body><p>Hello</p></body>
</html>"#;

    let sink = TestSink::new();
    let result = parse(html, sink).unwrap();

    println!("Events: {:?}", result.events);

    assert!(result.events.contains(&"doctype:html".to_string()));
    assert!(result.events.contains(&"start:html".to_string()));
    assert!(result.events.contains(&"start:head".to_string()));
    assert!(result.events.contains(&"start:body".to_string()));
    assert!(result.events.contains(&"text:Hello".to_string()));
}

#[test]
fn test_malformed_nesting() {
    let html = "<div><span><p></div></span></p>";
    let sink = TestSink::new();
    let result = parse(html, sink).unwrap();

    println!("Malformed events: {:?}", result.events);
    assert!(!result.events.is_empty());
}

#[test]
fn test_unclosed_tags() {
    let html = "<html><body><p>Text<div>More";
    let sink = TestSink::new();
    let result = parse(html, sink).unwrap();

    println!("Unclosed events: {:?}", result.events);
    assert!(result.events.contains(&"text:Text".to_string()));
}

#[test]
fn test_entity_decoding() {
    let html = "<p>&lt;div&gt; &amp; &quot;text&quot;</p>";
    let sink = TestSink::new();
    let result = parse(html, sink).unwrap();

    println!("Entity events: {:?}", result.events);
    assert!(result.events.iter().any(|e| e.contains("&")));
}

#[test]
fn test_nested_lists() {
    let html = "<ul><li>One<ul><li>Nested</li></ul></li><li>Two</li></ul>";
    let sink = TestSink::new();
    let result = parse(html, sink).unwrap();

    println!("Nested list events: {:?}", result.events);
    assert!(result.events.contains(&"start:ul".to_string()));
    assert!(result.events.contains(&"start:li".to_string()));
}

#[test]
fn test_table_structure() {
    let html = "<table><tr><td>Cell</td></tr></table>";
    let sink = TestSink::new();
    let result = parse(html, sink).unwrap();

    println!("Table events: {:?}", result.events);
    assert!(result.events.contains(&"start:table".to_string()));
    assert!(result.events.contains(&"start:tr".to_string()));
    assert!(result.events.contains(&"start:td".to_string()));
}

#[test]
fn test_comments() {
    let html = "<!-- comment 1 --><div><!-- comment 2 --></div>";
    let sink = TestSink::new();
    let result = parse(html, sink).unwrap();

    println!("Comment events: {:?}", result.events);
    assert!(result.events.iter().any(|e| e.starts_with("comment:")));
}

#[test]
fn test_attributes_with_special_chars() {
    let html = r#"<input type="text" data-value="foo&amp;bar" class="a b c">"#;
    let sink = TestSink::new();
    let result = parse(html, sink).unwrap();

    println!("Attribute events: {:?}", result.events);
    assert!(result.events.iter().any(|e| e.contains("type=text")));
}

// ============== P0 Tests: Script/Style/Entity Fixes ==============

#[test]
fn test_script_content_preserved() {
    // Script content should NOT be parsed as HTML - the < and > should be preserved as text
    let html = r#"<script>if (a < b && c > d) { alert("<div>"); }</script>"#;
    let sink = TestSink::new();
    let result = parse(html, sink).unwrap();

    println!("Script events: {:?}", result.events);

    // Should have script start and end tags
    assert!(result.events.contains(&"start:script".to_string()));
    assert!(result.events.contains(&"end:script".to_string()));

    // Should NOT have div element (it's inside the script as text)
    assert!(!result.events.iter().any(|e| e.contains("start:div")));

    // Script content should be preserved as text
    assert!(result.events.iter().any(|e| e.contains("if (a < b")));
}

#[test]
fn test_style_content_preserved() {
    // Style content should NOT be parsed as HTML
    let html = r#"<style>.foo > .bar { color: red; }</style>"#;
    let sink = TestSink::new();
    let result = parse(html, sink).unwrap();

    println!("Style events: {:?}", result.events);

    assert!(result.events.contains(&"start:style".to_string()));
    assert!(result.events.contains(&"end:style".to_string()));

    // CSS selectors with > should be preserved as text, not parsed as tags
    assert!(result.events.iter().any(|e| e.contains(".foo > .bar")));
}

#[test]
fn test_textarea_content_preserved() {
    // Textarea content should use RCDATA mode (entities decoded, but no tags)
    let html = r#"<textarea>Some <b>text</b> with &amp; entity</textarea>"#;
    let sink = TestSink::new();
    let result = parse(html, sink).unwrap();

    println!("Textarea events: {:?}", result.events);

    assert!(result.events.contains(&"start:textarea".to_string()));
    assert!(result.events.contains(&"end:textarea".to_string()));

    // <b> should NOT be parsed as a tag (it's text in textarea)
    assert!(!result.events.iter().any(|e| e == "start:b"));

    // Content should include the literal <b> text
    assert!(result.events.iter().any(|e| e.contains("<b>text</b>")));

    // Entity should be decoded
    assert!(result.events.iter().any(|e| e.contains("&")));
}

#[test]
fn test_title_content_preserved() {
    // Title content should use RCDATA mode
    let html = r#"<title>Page &amp; Title with <fake> tag</title>"#;
    let sink = TestSink::new();
    let result = parse(html, sink).unwrap();

    println!("Title events: {:?}", result.events);

    assert!(result.events.contains(&"start:title".to_string()));
    assert!(result.events.contains(&"end:title".to_string()));

    // <fake> should NOT be parsed as a tag
    assert!(!result.events.iter().any(|e| e.contains("start:fake")));
}

#[test]
fn test_entity_decoding_in_text() {
    // Entities in regular text content should be decoded
    let html = r#"<p>&lt;div&gt; means less than and greater than: &amp;</p>"#;
    let sink = TestSink::new();
    let result = parse(html, sink).unwrap();

    println!("Entity text events: {:?}", result.events);

    // The decoded text should contain < and > and &
    assert!(result.events.iter().any(|e| e.contains("<div>")));
    assert!(result.events.iter().any(|e| e.contains("&")));
}

#[test]
fn test_numeric_entity_in_text() {
    // Numeric entities should be decoded
    let html = r#"<p>&#65;&#66;&#67; and &#x41;&#x42;&#x43;</p>"#;
    let sink = TestSink::new();
    let result = parse(html, sink).unwrap();

    println!("Numeric entity events: {:?}", result.events);

    // &#65; = A, &#66; = B, &#67; = C
    // &#x41; = A, &#x42; = B, &#x43; = C
    assert!(result.events.iter().any(|e| e.contains("ABC")));
}

#[test]
fn test_script_with_closing_tag_in_string() {
    // Tricky case: </script> inside a string shouldn't close the script
    // Note: This is a simplified test - full spec compliance requires more complex handling
    let html = r#"<script>var x = "</script>"; // actual end</script>"#;
    let sink = TestSink::new();
    let result = parse(html, sink).unwrap();

    println!("Script string events: {:?}", result.events);

    // Should have at least one script element
    assert!(result.events.contains(&"start:script".to_string()));
}


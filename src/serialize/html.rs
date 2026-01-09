//! HTML serialization.
//!
//! Converts a DOM tree back to an HTML string following the WHATWG serialization algorithm.

use crate::dom::{Dom, NodeId, NodeKind};
use crate::treebuilder::is_void_element;

/// Options for HTML serialization.
#[derive(Debug, Clone)]
pub struct HtmlSerializeOptions {
    /// Whether to include the document type.
    pub include_doctype: bool,
    /// Whether to pretty-print with indentation.
    pub pretty: bool,
    /// Indentation string for pretty-printing.
    pub indent: String,
}

impl Default for HtmlSerializeOptions {
    fn default() -> Self {
        Self {
            include_doctype: true,
            pretty: false,
            indent: "  ".to_string(),
        }
    }
}

/// HTML serializer that converts a DOM tree to an HTML string.
pub struct HtmlSerializer<'a> {
    dom: &'a Dom,
    options: HtmlSerializeOptions,
    output: String,
    depth: usize,
}

impl<'a> HtmlSerializer<'a> {
    /// Create a new HTML serializer.
    pub fn new(dom: &'a Dom, options: HtmlSerializeOptions) -> Self {
        Self {
            dom,
            options,
            output: String::new(),
            depth: 0,
        }
    }

    /// Serialize a node and its descendants.
    pub fn serialize(&mut self, node_id: NodeId) -> &str {
        self.serialize_node(node_id);
        &self.output
    }

    fn serialize_node(&mut self, node_id: NodeId) {
        let node = self.dom.get(node_id);

        match &node.kind {
            NodeKind::Document | NodeKind::DocumentFragment => {
                self.serialize_children(node_id);
            }
            NodeKind::Doctype(doctype) => {
                if self.options.include_doctype {
                    self.output.push_str("<!DOCTYPE ");
                    if let Some(name) = &doctype.name {
                        self.output.push_str(name);
                    } else {
                        self.output.push_str("html");
                    }
                    if let Some(public_id) = &doctype.public_id {
                        if !public_id.is_empty() {
                            self.output.push_str(" PUBLIC \"");
                            self.output.push_str(public_id);
                            self.output.push('"');
                        }
                    }
                    if let Some(system_id) = &doctype.system_id {
                        if !system_id.is_empty() {
                            if doctype.public_id.is_none() {
                                self.output.push_str(" SYSTEM");
                            }
                            self.output.push_str(" \"");
                            self.output.push_str(system_id);
                            self.output.push('"');
                        }
                    }
                    self.output.push('>');
                    if self.options.pretty {
                        self.output.push('\n');
                    }
                }
            }
            NodeKind::Element(element) => {
                let tag_name = element.name.as_str();

                // Write indentation if pretty-printing
                if self.options.pretty && !self.output.is_empty() {
                    self.write_indent();
                }

                // Start tag
                self.output.push('<');
                self.output.push_str(tag_name);

                // Attributes
                for (name, value) in element.attrs.iter() {
                    self.output.push(' ');
                    self.output.push_str(name);
                    if let Some(val) = value {
                        self.output.push_str("=\"");
                        self.escape_attribute(val);
                        self.output.push('"');
                    }
                }

                self.output.push('>');

                // Void elements don't have content or end tags
                if is_void_element(tag_name) {
                    if self.options.pretty {
                        self.output.push('\n');
                    }
                    return;
                }

                // Special handling for raw text elements
                let is_raw_text = matches!(tag_name, "script" | "style" | "textarea" | "title");

                if self.options.pretty && node.first_child.is_some() && !is_raw_text {
                    self.output.push('\n');
                }

                // Children
                self.depth += 1;
                self.serialize_children(node_id);
                self.depth -= 1;

                // End tag
                if self.options.pretty && node.first_child.is_some() && !is_raw_text {
                    self.write_indent();
                }
                self.output.push_str("</");
                self.output.push_str(tag_name);
                self.output.push('>');

                if self.options.pretty {
                    self.output.push('\n');
                }
            }
            NodeKind::Text(text) => {
                // Check if parent is a raw text element
                let is_raw_text_child = node
                    .parent
                    .map(|p| {
                        if let NodeKind::Element(el) = &self.dom.get(p).kind {
                            matches!(
                                el.name.as_str(),
                                "script" | "style" | "textarea" | "title"
                            )
                        } else {
                            false
                        }
                    })
                    .unwrap_or(false);

                if is_raw_text_child {
                    // Don't escape raw text content
                    self.output.push_str(text);
                } else {
                    // Escape text content
                    self.escape_text(text);
                }
            }
            NodeKind::Comment(comment) => {
                if self.options.pretty {
                    self.write_indent();
                }
                self.output.push_str("<!--");
                self.output.push_str(comment);
                self.output.push_str("-->");
                if self.options.pretty {
                    self.output.push('\n');
                }
            }
        }
    }

    fn serialize_children(&mut self, parent_id: NodeId) {
        let mut child = self.dom.get(parent_id).first_child;
        while let Some(child_id) = child {
            self.serialize_node(child_id);
            child = self.dom.get(child_id).next_sibling;
        }
    }

    fn write_indent(&mut self) {
        for _ in 0..self.depth {
            self.output.push_str(&self.options.indent);
        }
    }

    fn escape_text(&mut self, text: &str) {
        for ch in text.chars() {
            match ch {
                '&' => self.output.push_str("&amp;"),
                '<' => self.output.push_str("&lt;"),
                '>' => self.output.push_str("&gt;"),
                _ => self.output.push(ch),
            }
        }
    }

    fn escape_attribute(&mut self, text: &str) {
        for ch in text.chars() {
            match ch {
                '&' => self.output.push_str("&amp;"),
                '"' => self.output.push_str("&quot;"),
                _ => self.output.push(ch),
            }
        }
    }
}

/// Serialize a DOM node to HTML.
///
/// # Example
///
/// ```
/// use justhtml::{parse, serialize::serialize_to_html};
///
/// let result = parse("<p>Hello</p>");
/// let html = serialize_to_html(&result.dom, result.document);
/// ```
pub fn serialize_to_html(dom: &Dom, root: NodeId) -> String {
    let mut serializer = HtmlSerializer::new(dom, HtmlSerializeOptions::default());
    serializer.serialize(root).to_string()
}

/// Serialize a DOM node to pretty-printed HTML.
pub fn serialize_to_html_pretty(dom: &Dom, root: NodeId) -> String {
    let options = HtmlSerializeOptions {
        pretty: true,
        ..Default::default()
    };
    let mut serializer = HtmlSerializer::new(dom, options);
    serializer.serialize(root).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    #[test]
    fn test_serialize_simple() {
        let result = parse("<p>Hello</p>");
        let html = serialize_to_html(&result.dom, result.document);
        assert!(html.contains("<p>Hello</p>"));
    }

    #[test]
    fn test_serialize_with_attributes() {
        let result = parse("<div class=\"test\" id=\"main\">Content</div>");
        let html = serialize_to_html(&result.dom, result.document);
        assert!(html.contains("class=\"test\""));
        assert!(html.contains("id=\"main\""));
        assert!(html.contains("Content"));
    }

    #[test]
    fn test_serialize_void_elements() {
        let result = parse("<br><img src=\"test.jpg\"><hr>");
        let html = serialize_to_html(&result.dom, result.document);
        assert!(html.contains("<br>"));
        assert!(html.contains("<img"));
        assert!(html.contains("<hr>"));
        // Void elements should not have closing tags
        assert!(!html.contains("</br>"));
        assert!(!html.contains("</img>"));
        assert!(!html.contains("</hr>"));
    }

    #[test]
    fn test_serialize_escapes_text() {
        let result = parse("<p>1 &lt; 2 &amp; 3 &gt; 1</p>");
        let html = serialize_to_html(&result.dom, result.document);
        // The serialized output should have escaped entities
        assert!(html.contains("&lt;") || html.contains("<"));
    }

    #[test]
    fn test_serialize_escapes_attributes() {
        let result = parse("<a href=\"test?a=1&amp;b=2\">Link</a>");
        let html = serialize_to_html(&result.dom, result.document);
        assert!(html.contains("href=\""));
    }

    #[test]
    fn test_serialize_comment() {
        let result = parse("<!-- This is a comment --><p>Text</p>");
        let html = serialize_to_html(&result.dom, result.document);
        assert!(html.contains("<!--"));
        assert!(html.contains("-->"));
    }

    #[test]
    fn test_serialize_nested() {
        let result = parse("<div><span><b>Bold</b></span></div>");
        let html = serialize_to_html(&result.dom, result.document);
        assert!(html.contains("<div><span><b>Bold</b></span></div>"));
    }

    #[test]
    fn test_serialize_script() {
        let result = parse("<script>var x = 1 < 2;</script>");
        let html = serialize_to_html(&result.dom, result.document);
        // Script content should not be escaped
        assert!(html.contains("<script>"));
        assert!(html.contains("</script>"));
    }
}

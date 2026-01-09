//! Markdown serialization.
//!
//! Converts a DOM tree to a Markdown string.

use crate::dom::{Dom, NodeId, NodeKind};

/// Options for Markdown serialization.
#[derive(Debug, Clone)]
pub struct MarkdownSerializeOptions {
    /// Use ATX-style headers (# Header) vs Setext-style (underlined).
    pub atx_headers: bool,
    /// Character for unordered list items (-, *, +).
    pub bullet_char: char,
    /// Character for emphasis (* or _).
    pub emphasis_char: char,
    /// Character for strong emphasis (** or __).
    pub strong_char: &'static str,
    /// Code fence character (` or ~).
    pub code_fence: &'static str,
}

impl Default for MarkdownSerializeOptions {
    fn default() -> Self {
        Self {
            atx_headers: true,
            bullet_char: '-',
            emphasis_char: '*',
            strong_char: "**",
            code_fence: "```",
        }
    }
}

/// Markdown serializer that converts a DOM tree to a Markdown string.
pub struct MarkdownSerializer<'a> {
    dom: &'a Dom,
    options: MarkdownSerializeOptions,
    output: String,
    list_depth: usize,
    ordered_list_counters: Vec<usize>,
    in_pre: bool,
}

impl<'a> MarkdownSerializer<'a> {
    /// Create a new Markdown serializer.
    pub fn new(dom: &'a Dom, options: MarkdownSerializeOptions) -> Self {
        Self {
            dom,
            options,
            output: String::new(),
            list_depth: 0,
            ordered_list_counters: Vec::new(),
            in_pre: false,
        }
    }

    /// Serialize a node and its descendants to Markdown.
    pub fn serialize(&mut self, node_id: NodeId) -> &str {
        self.serialize_node(node_id);
        // Clean up excessive newlines
        self.cleanup_output();
        &self.output
    }

    fn serialize_node(&mut self, node_id: NodeId) {
        let node = self.dom.get(node_id);

        match &node.kind {
            NodeKind::Document | NodeKind::DocumentFragment => {
                self.serialize_children(node_id);
            }
            NodeKind::Doctype(_) => {
                // Skip doctype in markdown
            }
            NodeKind::Element(element) => {
                let tag_name = element.name.as_str();

                match tag_name {
                    // Headings
                    "h1" => self.serialize_heading(node_id, 1),
                    "h2" => self.serialize_heading(node_id, 2),
                    "h3" => self.serialize_heading(node_id, 3),
                    "h4" => self.serialize_heading(node_id, 4),
                    "h5" => self.serialize_heading(node_id, 5),
                    "h6" => self.serialize_heading(node_id, 6),

                    // Paragraphs
                    "p" => {
                        self.ensure_blank_line();
                        self.serialize_children(node_id);
                        self.ensure_blank_line();
                    }

                    // Inline formatting
                    "b" | "strong" => {
                        self.output.push_str(self.options.strong_char);
                        self.serialize_children(node_id);
                        self.output.push_str(self.options.strong_char);
                    }
                    "i" | "em" => {
                        self.output.push(self.options.emphasis_char);
                        self.serialize_children(node_id);
                        self.output.push(self.options.emphasis_char);
                    }
                    "u" => {
                        // Markdown doesn't have underline, use emphasis
                        self.output.push(self.options.emphasis_char);
                        self.serialize_children(node_id);
                        self.output.push(self.options.emphasis_char);
                    }
                    "s" | "strike" | "del" => {
                        self.output.push_str("~~");
                        self.serialize_children(node_id);
                        self.output.push_str("~~");
                    }
                    "code" if !self.in_pre => {
                        self.output.push('`');
                        self.serialize_children(node_id);
                        self.output.push('`');
                    }
                    "code" => {
                        // Inside pre, just output content
                        self.serialize_children(node_id);
                    }

                    // Links and images
                    "a" => {
                        let href = element
                            .attrs
                            .get("href")
                            .flatten()
                            .unwrap_or("");
                        self.output.push('[');
                        self.serialize_children(node_id);
                        self.output.push_str("](");
                        self.output.push_str(href);
                        self.output.push(')');
                    }
                    "img" => {
                        let src = element
                            .attrs
                            .get("src")
                            .flatten()
                            .unwrap_or("");
                        let alt = element
                            .attrs
                            .get("alt")
                            .flatten()
                            .unwrap_or("");
                        self.output.push_str("![");
                        self.output.push_str(alt);
                        self.output.push_str("](");
                        self.output.push_str(src);
                        self.output.push(')');
                    }

                    // Lists
                    "ul" => {
                        self.ensure_newline();
                        self.list_depth += 1;
                        self.serialize_children(node_id);
                        self.list_depth -= 1;
                        self.ensure_newline();
                    }
                    "ol" => {
                        self.ensure_newline();
                        self.list_depth += 1;
                        self.ordered_list_counters.push(1);
                        self.serialize_children(node_id);
                        self.ordered_list_counters.pop();
                        self.list_depth -= 1;
                        self.ensure_newline();
                    }
                    "li" => {
                        self.ensure_newline();
                        // Indent for nested lists
                        for _ in 1..self.list_depth {
                            self.output.push_str("  ");
                        }
                        // Check if we're in an ordered list
                        if let Some(counter) = self.ordered_list_counters.last_mut() {
                            self.output.push_str(&format!("{}. ", counter));
                            *counter += 1;
                        } else {
                            self.output.push(self.options.bullet_char);
                            self.output.push(' ');
                        }
                        self.serialize_children(node_id);
                    }

                    // Block elements
                    "blockquote" => {
                        self.ensure_blank_line();
                        let content = self.get_text_content(node_id);
                        for line in content.lines() {
                            self.output.push_str("> ");
                            self.output.push_str(line);
                            self.output.push('\n');
                        }
                        self.ensure_newline();
                    }
                    "pre" => {
                        self.ensure_blank_line();
                        self.output.push_str(self.options.code_fence);
                        self.output.push('\n');
                        self.in_pre = true;
                        self.serialize_children(node_id);
                        self.in_pre = false;
                        self.ensure_newline();
                        self.output.push_str(self.options.code_fence);
                        self.ensure_blank_line();
                    }
                    "hr" => {
                        self.ensure_blank_line();
                        self.output.push_str("---");
                        self.ensure_blank_line();
                    }
                    "br" => {
                        self.output.push_str("  \n");
                    }

                    // Block containers
                    "div" | "section" | "article" | "main" | "aside" | "header" | "footer"
                    | "nav" => {
                        self.ensure_newline();
                        self.serialize_children(node_id);
                        self.ensure_newline();
                    }

                    // Inline/other - just serialize children
                    "span" | "html" | "head" | "body" | "title" | "meta" | "link" | "script"
                    | "style" | "noscript" => {
                        self.serialize_children(node_id);
                    }

                    // Tables (basic support)
                    "table" => {
                        self.ensure_blank_line();
                        self.serialize_children(node_id);
                        self.ensure_blank_line();
                    }
                    "thead" | "tbody" | "tfoot" => {
                        self.serialize_children(node_id);
                    }
                    "tr" => {
                        self.output.push('|');
                        self.serialize_children(node_id);
                        self.output.push('\n');
                    }
                    "th" | "td" => {
                        self.output.push(' ');
                        self.serialize_children(node_id);
                        self.output.push_str(" |");
                    }

                    // Default: serialize children
                    _ => {
                        self.serialize_children(node_id);
                    }
                }
            }
            NodeKind::Text(text) => {
                if self.in_pre {
                    self.output.push_str(text);
                } else {
                    // Normalize whitespace for non-pre text
                    let normalized = self.normalize_whitespace(text);
                    self.output.push_str(&normalized);
                }
            }
            NodeKind::Comment(_) => {
                // Skip comments in markdown
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

    fn serialize_heading(&mut self, node_id: NodeId, level: usize) {
        self.ensure_blank_line();
        if self.options.atx_headers {
            for _ in 0..level {
                self.output.push('#');
            }
            self.output.push(' ');
            self.serialize_children(node_id);
            self.ensure_blank_line();
        } else {
            self.serialize_children(node_id);
            self.output.push('\n');
            let underline = if level == 1 { '=' } else { '-' };
            for _ in 0..20 {
                self.output.push(underline);
            }
            self.ensure_blank_line();
        }
    }

    fn get_text_content(&self, node_id: NodeId) -> String {
        let mut result = String::new();
        self.collect_text(node_id, &mut result);
        result
    }

    fn collect_text(&self, node_id: NodeId, result: &mut String) {
        let node = self.dom.get(node_id);
        match &node.kind {
            NodeKind::Text(text) => result.push_str(text),
            _ => {
                let mut child = node.first_child;
                while let Some(child_id) = child {
                    self.collect_text(child_id, result);
                    child = self.dom.get(child_id).next_sibling;
                }
            }
        }
    }

    fn normalize_whitespace(&self, text: &str) -> String {
        let mut result = String::new();
        let mut last_was_space = false;

        for ch in text.chars() {
            if ch.is_whitespace() {
                if !last_was_space && !result.is_empty() {
                    result.push(' ');
                    last_was_space = true;
                }
            } else {
                result.push(ch);
                last_was_space = false;
            }
        }

        result
    }

    fn ensure_newline(&mut self) {
        if !self.output.is_empty() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }
    }

    fn ensure_blank_line(&mut self) {
        if self.output.is_empty() {
            return;
        }
        if !self.output.ends_with('\n') {
            self.output.push_str("\n\n");
        } else if !self.output.ends_with("\n\n") {
            self.output.push('\n');
        }
    }

    fn cleanup_output(&mut self) {
        // Remove leading/trailing whitespace
        let trimmed = self.output.trim().to_string();
        self.output = trimmed;

        // Ensure single trailing newline
        if !self.output.is_empty() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }
    }
}

/// Serialize a DOM node to Markdown.
///
/// # Example
///
/// ```
/// use whatwg_html_rs::{parse, serialize::serialize_to_markdown};
///
/// let result = parse("<h1>Title</h1><p>Hello <b>world</b>!</p>");
/// let md = serialize_to_markdown(&result.dom, result.document);
/// assert!(md.contains("# Title"));
/// assert!(md.contains("**world**"));
/// ```
pub fn serialize_to_markdown(dom: &Dom, root: NodeId) -> String {
    let mut serializer = MarkdownSerializer::new(dom, MarkdownSerializeOptions::default());
    serializer.serialize(root).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    #[test]
    fn test_headings() {
        let result = parse("<h1>Title</h1>");
        let md = serialize_to_markdown(&result.dom, result.document);
        assert!(md.contains("# Title"));
    }

    #[test]
    fn test_heading_levels() {
        let result = parse("<h1>H1</h1><h2>H2</h2><h3>H3</h3>");
        let md = serialize_to_markdown(&result.dom, result.document);
        assert!(md.contains("# H1"));
        assert!(md.contains("## H2"));
        assert!(md.contains("### H3"));
    }

    #[test]
    fn test_paragraph() {
        let result = parse("<p>Hello world</p>");
        let md = serialize_to_markdown(&result.dom, result.document);
        assert!(md.contains("Hello world"));
    }

    #[test]
    fn test_bold() {
        let result = parse("<p>Hello <b>world</b>!</p>");
        let md = serialize_to_markdown(&result.dom, result.document);
        assert!(md.contains("**world**"));
    }

    #[test]
    fn test_italic() {
        let result = parse("<p>Hello <i>world</i>!</p>");
        let md = serialize_to_markdown(&result.dom, result.document);
        assert!(md.contains("*world*"));
    }

    #[test]
    fn test_link() {
        let result = parse("<a href=\"https://example.com\">Click here</a>");
        let md = serialize_to_markdown(&result.dom, result.document);
        assert!(md.contains("[Click here](https://example.com)"));
    }

    #[test]
    fn test_image() {
        let result = parse("<img src=\"image.png\" alt=\"My image\">");
        let md = serialize_to_markdown(&result.dom, result.document);
        assert!(md.contains("![My image](image.png)"));
    }

    #[test]
    fn test_unordered_list() {
        let result = parse("<ul><li>Item 1</li><li>Item 2</li></ul>");
        let md = serialize_to_markdown(&result.dom, result.document);
        assert!(md.contains("- Item 1"));
        assert!(md.contains("- Item 2"));
    }

    #[test]
    fn test_ordered_list() {
        let result = parse("<ol><li>First</li><li>Second</li></ol>");
        let md = serialize_to_markdown(&result.dom, result.document);
        assert!(md.contains("1. First"));
        assert!(md.contains("2. Second"));
    }

    #[test]
    fn test_code_inline() {
        let result = parse("<p>Use <code>println!</code> to print</p>");
        let md = serialize_to_markdown(&result.dom, result.document);
        assert!(md.contains("`println!`"));
    }

    #[test]
    fn test_code_block() {
        let result = parse("<pre><code>fn main() {}</code></pre>");
        let md = serialize_to_markdown(&result.dom, result.document);
        assert!(md.contains("```"));
        assert!(md.contains("fn main() {}"));
    }

    #[test]
    fn test_blockquote() {
        let result = parse("<blockquote>Famous quote</blockquote>");
        let md = serialize_to_markdown(&result.dom, result.document);
        assert!(md.contains("> Famous quote"));
    }

    #[test]
    fn test_horizontal_rule() {
        let result = parse("<p>Above</p><hr><p>Below</p>");
        let md = serialize_to_markdown(&result.dom, result.document);
        assert!(md.contains("---"));
    }

    #[test]
    fn test_strikethrough() {
        let result = parse("<p>This is <del>deleted</del> text</p>");
        let md = serialize_to_markdown(&result.dom, result.document);
        assert!(md.contains("~~deleted~~"));
    }

    #[test]
    fn test_complex_document() {
        let html = r#"
            <h1>My Document</h1>
            <p>This is a <b>bold</b> and <i>italic</i> paragraph.</p>
            <h2>Links</h2>
            <p>Visit <a href="https://rust-lang.org">Rust</a>.</p>
            <h2>List</h2>
            <ul>
                <li>Item 1</li>
                <li>Item 2</li>
            </ul>
        "#;
        let result = parse(html);
        let md = serialize_to_markdown(&result.dom, result.document);

        assert!(md.contains("# My Document"));
        assert!(md.contains("**bold**"));
        assert!(md.contains("*italic*"));
        assert!(md.contains("## Links"));
        assert!(md.contains("[Rust](https://rust-lang.org)"));
        assert!(md.contains("- Item 1"));
    }
}

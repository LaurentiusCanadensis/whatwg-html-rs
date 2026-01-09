//! JustHTML - A WHATWG HTML5 spec-compliant parser
//!
//! This crate provides a pure Rust implementation of the HTML5 parsing algorithm
//! as specified by WHATWG. It includes:
//!
//! - Full HTML5 tokenization
//! - Tree construction with error recovery
//! - HTML sanitization with security-focused policies
//! - CSS selector support for DOM querying
//! - Multiple output formats
//!
//! # Example
//!
//! ```
//! use justhtml::{parse, Tokenizer};
//!
//! // Parse HTML into a DOM tree
//! let result = parse("<html><body><p>Hello World</p></body></html>");
//! println!("DOM has {} nodes", result.dom.len());
//!
//! // Or use the tokenizer directly
//! let html = "<div class=\"container\"><p>Hello</p></div>";
//! let tokenizer = Tokenizer::new(html);
//! for token in tokenizer {
//!     println!("{:?}", token);
//! }
//! ```

pub mod dom;
pub mod error;
pub mod sanitize;
pub mod selector;
pub mod serialize;
pub mod tokenizer;
pub mod treebuilder;

// Re-export main types
pub use dom::{Dom, Element, Namespace, Node, NodeId, NodeKind};
pub use error::{ParseError, SelectorError, UnsafeHtmlError};
pub use tokenizer::{Token, Tokenizer};
pub use treebuilder::TreeBuilder;

/// Result of parsing HTML.
pub struct ParseResult {
    /// The DOM tree.
    pub dom: Dom,
    /// The document root node ID.
    pub document: NodeId,
    /// Parse errors encountered.
    pub errors: Vec<ParseError>,
}

impl ParseResult {
    /// Get a node wrapper for the document root.
    pub fn root(&self) -> Node<'_> {
        Node::new(&self.dom, self.document)
    }

    /// Serialize the DOM back to HTML.
    pub fn to_html(&self) -> String {
        serialize::serialize_to_html(&self.dom, self.document)
    }
}

/// Parse HTML and return a DOM tree.
///
/// This is a convenience function that parses a complete HTML document.
///
/// # Example
///
/// ```
/// let result = justhtml::parse("<html><body>Hello</body></html>");
/// println!("Document has {} nodes", result.dom.len());
/// ```
pub fn parse(html: &str) -> ParseResult {
    let builder = TreeBuilder::new(html);
    let (dom, document, errors) = builder.parse();
    ParseResult {
        dom,
        document,
        errors,
    }
}

/// Parse HTML with error collection enabled.
///
/// # Example
///
/// ```
/// let result = justhtml::parse_with_errors("<p>Unclosed paragraph");
/// for error in &result.errors {
///     println!("Error: {}", error);
/// }
/// ```
pub fn parse_with_errors(html: &str) -> ParseResult {
    let builder = TreeBuilder::new(html).with_errors();
    let (dom, document, errors) = builder.parse();
    ParseResult {
        dom,
        document,
        errors,
    }
}

/// Parse an HTML fragment with a context element.
///
/// This is useful for parsing partial HTML that will be inserted into
/// an existing document.
///
/// # Example
///
/// ```ignore
/// let result = justhtml::parse_fragment("<li>Item 1</li><li>Item 2</li>", "ul");
/// ```
pub fn parse_fragment(_html: &str, _context: &str) -> ParseResult {
    // TODO: Implement fragment parsing with context
    let builder = TreeBuilder::new(_html);
    let (dom, document, errors) = builder.parse();
    ParseResult {
        dom,
        document,
        errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer_integration() {
        let html = "<div><p>Hello</p></div>";
        let tokenizer = Tokenizer::new(html);
        let tokens: Vec<_> = tokenizer.collect();

        // Should have: <div>, <p>, "Hello", </p>, </div>
        assert_eq!(tokens.len(), 5);
    }

    #[test]
    fn test_parse_simple() {
        let result = parse("<p>Hello</p>");
        assert!(result.dom.len() > 0);
    }

    #[test]
    fn test_parse_with_errors() {
        let result = parse_with_errors("<div><p>Unclosed");
        // Should parse without panicking
        assert!(result.dom.len() > 0);
    }

    #[test]
    fn test_full_document() {
        let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Test Page</title>
</head>
<body>
    <h1>Hello World</h1>
    <p>This is a <strong>test</strong>.</p>
</body>
</html>"#;

        let result = parse(html);
        assert!(result.dom.len() > 5);
    }
}

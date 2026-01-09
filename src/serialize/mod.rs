//! Serialization module.
//!
//! Provides functionality to convert a DOM tree to HTML or Markdown strings.

mod html;
mod markdown;

pub use html::{serialize_to_html, serialize_to_html_pretty, HtmlSerializer, HtmlSerializeOptions};
pub use markdown::{serialize_to_markdown, MarkdownSerializer, MarkdownSerializeOptions};

//! HTML serialization module.
//!
//! Provides functionality to convert a DOM tree back to HTML strings.

mod html;

pub use html::{serialize_to_html, HtmlSerializer};

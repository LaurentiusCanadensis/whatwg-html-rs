//! HTML sanitization module.
//!
//! Provides security-focused HTML cleaning to prevent XSS attacks.
//! The sanitizer uses an allow-list approach for tags, attributes, and URL schemes.

mod policy;
mod sanitizer;

pub use policy::{
    SanitizationPolicy, UrlHandling, UrlPolicy, UrlProxy, UrlRule, UnsafeHandling,
    DEFAULT_POLICY,
};
pub use sanitizer::{sanitize_dom, Sanitizer};

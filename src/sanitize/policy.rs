//! Sanitization policy types.
//!
//! Defines the allow-list policy for HTML sanitization.

use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};

/// How to handle unsafe HTML constructs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsafeHandling {
    /// Remove unsafe constructs silently (default).
    Strip,
    /// Raise an error on unsafe constructs.
    Raise,
    /// Collect errors but continue processing.
    Collect,
}

impl Default for UnsafeHandling {
    fn default() -> Self {
        Self::Strip
    }
}

/// How to handle URL attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UrlHandling {
    /// Keep the URL as-is.
    Allow,
    /// Remove the attribute.
    Strip,
    /// Rewrite through a proxy.
    Proxy,
}

impl Default for UrlHandling {
    fn default() -> Self {
        Self::Strip
    }
}

/// Configuration for URL proxying.
#[derive(Debug, Clone)]
pub struct UrlProxy {
    /// The proxy URL base.
    pub url: String,
    /// Query parameter name for the original URL.
    pub param: String,
}

impl UrlProxy {
    /// Create a new URL proxy configuration.
    pub fn new(url: impl Into<String>, param: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            param: param.into(),
        }
    }

    /// Rewrite a URL through this proxy.
    pub fn rewrite(&self, original: &str) -> String {
        let sep = if self.url.contains('?') { '&' } else { '?' };
        format!(
            "{}{sep}{}={}",
            self.url,
            self.param,
            urlencoding::encode(original)
        )
    }
}

/// Rule for URL-valued attributes.
#[derive(Debug, Clone)]
pub struct UrlRule {
    /// Allow same-document fragments (#foo).
    pub allow_fragment: bool,
    /// Resolve protocol-relative URLs to this scheme.
    pub resolve_protocol_relative: Option<String>,
    /// Allowed URL schemes (lowercase).
    pub allowed_schemes: HashSet<String>,
    /// Allowed hosts (if Some, only these hosts are allowed).
    pub allowed_hosts: Option<HashSet<String>>,
    /// Per-rule handling override.
    pub handling: Option<UrlHandling>,
    /// Per-rule relative URL allowance.
    pub allow_relative: Option<bool>,
    /// Per-rule proxy override.
    pub proxy: Option<UrlProxy>,
}

impl Default for UrlRule {
    fn default() -> Self {
        Self {
            allow_fragment: true,
            resolve_protocol_relative: Some("https".to_string()),
            allowed_schemes: HashSet::new(),
            allowed_hosts: None,
            handling: None,
            allow_relative: None,
            proxy: None,
        }
    }
}

impl UrlRule {
    /// Create a new URL rule with allowed schemes.
    pub fn with_schemes<I, S>(schemes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            allowed_schemes: schemes.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }
}

/// Policy for URL handling.
#[derive(Debug, Clone)]
pub struct UrlPolicy {
    /// Default handling for URL attributes.
    pub default_handling: UrlHandling,
    /// Default allowance for relative URLs.
    pub default_allow_relative: bool,
    /// Rules for specific (tag, attribute) pairs.
    pub allow_rules: HashMap<(String, String), UrlRule>,
    /// Default proxy configuration.
    pub proxy: Option<UrlProxy>,
}

impl Default for UrlPolicy {
    fn default() -> Self {
        Self {
            default_handling: UrlHandling::Strip,
            default_allow_relative: true,
            allow_rules: HashMap::new(),
            proxy: None,
        }
    }
}

impl UrlPolicy {
    /// Get the rule for a specific tag/attribute pair.
    pub fn get_rule(&self, tag: &str, attr: &str) -> Option<&UrlRule> {
        self.allow_rules
            .get(&(tag.to_ascii_lowercase(), attr.to_ascii_lowercase()))
    }

    /// Check if a URL is allowed by the given rule.
    pub fn check_url(&self, rule: &UrlRule, url: &str) -> bool {
        let url = url.trim();

        // Empty URLs are always invalid
        if url.is_empty() {
            return false;
        }

        // Check for fragments
        if url.starts_with('#') {
            return rule.allow_fragment;
        }

        // Check for relative URLs
        if !url.contains(':') && !url.starts_with("//") {
            return rule.allow_relative.unwrap_or(self.default_allow_relative);
        }

        // Handle protocol-relative URLs
        if url.starts_with("//") {
            if let Some(ref scheme) = rule.resolve_protocol_relative {
                let full_url = format!("{scheme}:{url}");
                return self.check_absolute_url(rule, &full_url);
            }
            return false;
        }

        // Check absolute URL
        self.check_absolute_url(rule, url)
    }

    fn check_absolute_url(&self, rule: &UrlRule, url: &str) -> bool {
        // Extract scheme
        if let Some(colon_pos) = url.find(':') {
            let scheme = url[..colon_pos].to_ascii_lowercase();
            if !rule.allowed_schemes.contains(&scheme) {
                return false;
            }

            // Check host if required
            if let Some(ref allowed_hosts) = rule.allowed_hosts {
                if let Some(host) = extract_host(url) {
                    if !allowed_hosts.contains(&host.to_ascii_lowercase()) {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            true
        } else {
            false
        }
    }
}

/// Extract host from a URL.
fn extract_host(url: &str) -> Option<&str> {
    // Skip scheme
    let after_scheme = url.find("://").map(|p| &url[p + 3..])?;

    // Find end of host (port, path, query, or fragment)
    let host_end = after_scheme
        .find(|c| c == '/' || c == ':' || c == '?' || c == '#')
        .unwrap_or(after_scheme.len());

    let host = &after_scheme[..host_end];
    if host.is_empty() {
        None
    } else {
        Some(host)
    }
}

/// Sanitization policy defining what HTML constructs are allowed.
#[derive(Debug, Clone)]
pub struct SanitizationPolicy {
    /// Allowed tag names (lowercase).
    pub allowed_tags: HashSet<String>,
    /// Allowed attributes per tag. "*" matches all tags.
    pub allowed_attributes: HashMap<String, HashSet<String>>,
    /// URL handling policy.
    pub url_policy: UrlPolicy,
    /// Whether to drop HTML comments.
    pub drop_comments: bool,
    /// Whether to drop DOCTYPE.
    pub drop_doctype: bool,
    /// Whether to drop SVG/MathML content.
    pub drop_foreign_namespaces: bool,
    /// Whether to keep children of disallowed tags.
    pub strip_disallowed_tags: bool,
    /// Tags whose content should also be dropped.
    pub drop_content_tags: HashSet<String>,
    /// Allowed CSS properties for style attributes.
    pub allowed_css_properties: HashSet<String>,
    /// Required rel tokens for links.
    pub force_link_rel: HashSet<String>,
    /// How to handle unsafe constructs.
    pub unsafe_handling: UnsafeHandling,
}

impl Default for SanitizationPolicy {
    fn default() -> Self {
        DEFAULT_POLICY.clone()
    }
}

impl SanitizationPolicy {
    /// Create a new empty policy (blocks everything).
    pub fn empty() -> Self {
        Self {
            allowed_tags: HashSet::new(),
            allowed_attributes: HashMap::new(),
            url_policy: UrlPolicy::default(),
            drop_comments: true,
            drop_doctype: true,
            drop_foreign_namespaces: true,
            strip_disallowed_tags: true,
            drop_content_tags: ["script", "style"].iter().map(|s| s.to_string()).collect(),
            allowed_css_properties: HashSet::new(),
            force_link_rel: HashSet::new(),
            unsafe_handling: UnsafeHandling::Strip,
        }
    }

    /// Check if a tag is allowed.
    pub fn is_tag_allowed(&self, tag: &str) -> bool {
        self.allowed_tags.contains(&tag.to_ascii_lowercase())
    }

    /// Check if an attribute is allowed for a tag.
    pub fn is_attribute_allowed(&self, tag: &str, attr: &str) -> bool {
        let tag_lower = tag.to_ascii_lowercase();
        let attr_lower = attr.to_ascii_lowercase();

        // Check global attributes first
        if let Some(global_attrs) = self.allowed_attributes.get("*") {
            if global_attrs.contains(&attr_lower) {
                return true;
            }
        }

        // Check tag-specific attributes
        if let Some(tag_attrs) = self.allowed_attributes.get(&tag_lower) {
            return tag_attrs.contains(&attr_lower);
        }

        false
    }

    /// Get the effective allowed attributes for a tag.
    pub fn allowed_attrs_for_tag(&self, tag: &str) -> HashSet<String> {
        let tag_lower = tag.to_ascii_lowercase();
        let mut attrs = HashSet::new();

        // Add global attributes
        if let Some(global_attrs) = self.allowed_attributes.get("*") {
            attrs.extend(global_attrs.iter().cloned());
        }

        // Add tag-specific attributes
        if let Some(tag_attrs) = self.allowed_attributes.get(&tag_lower) {
            attrs.extend(tag_attrs.iter().cloned());
        }

        attrs
    }

    /// Check if a tag's content should be dropped.
    pub fn should_drop_content(&self, tag: &str) -> bool {
        self.drop_content_tags.contains(&tag.to_ascii_lowercase())
    }
}

/// Default sanitization policy.
pub static DEFAULT_POLICY: Lazy<SanitizationPolicy> = Lazy::new(|| {
    let mut allowed_tags = HashSet::new();
    for tag in [
        // Text / structure
        "p", "br", "div", "span", "blockquote", "pre", "code",
        // Headings
        "h1", "h2", "h3", "h4", "h5", "h6",
        // Lists
        "ul", "ol", "li",
        // Tables
        "table", "thead", "tbody", "tfoot", "tr", "th", "td",
        // Text formatting
        "b", "strong", "i", "em", "u", "s", "sub", "sup", "small", "mark",
        // Line breaks
        "hr",
        // Links and images
        "a", "img",
    ] {
        allowed_tags.insert(tag.to_string());
    }

    let mut allowed_attributes: HashMap<String, HashSet<String>> = HashMap::new();
    allowed_attributes.insert(
        "*".to_string(),
        ["class", "id", "title", "lang", "dir"]
            .iter()
            .map(|s| s.to_string())
            .collect(),
    );
    allowed_attributes.insert(
        "a".to_string(),
        ["href", "title"].iter().map(|s| s.to_string()).collect(),
    );
    allowed_attributes.insert(
        "img".to_string(),
        ["src", "alt", "title", "width", "height", "loading", "decoding"]
            .iter()
            .map(|s| s.to_string())
            .collect(),
    );
    allowed_attributes.insert(
        "th".to_string(),
        ["colspan", "rowspan"]
            .iter()
            .map(|s| s.to_string())
            .collect(),
    );
    allowed_attributes.insert(
        "td".to_string(),
        ["colspan", "rowspan"]
            .iter()
            .map(|s| s.to_string())
            .collect(),
    );

    let mut url_rules = HashMap::new();
    url_rules.insert(
        ("a".to_string(), "href".to_string()),
        UrlRule {
            allowed_schemes: ["http", "https", "mailto", "tel"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            resolve_protocol_relative: Some("https".to_string()),
            ..Default::default()
        },
    );
    url_rules.insert(
        ("img".to_string(), "src".to_string()),
        UrlRule {
            allowed_schemes: HashSet::new(), // Block remote images by default
            resolve_protocol_relative: None,
            ..Default::default()
        },
    );

    SanitizationPolicy {
        allowed_tags,
        allowed_attributes,
        url_policy: UrlPolicy {
            default_handling: UrlHandling::Allow,
            default_allow_relative: true,
            allow_rules: url_rules,
            proxy: None,
        },
        drop_comments: true,
        drop_doctype: true,
        drop_foreign_namespaces: true,
        strip_disallowed_tags: true,
        drop_content_tags: ["script", "style"].iter().map(|s| s.to_string()).collect(),
        allowed_css_properties: HashSet::new(),
        force_link_rel: HashSet::new(),
        unsafe_handling: UnsafeHandling::Strip,
    }
});

/// Conservative CSS properties preset for text styling.
pub static CSS_PRESET_TEXT: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "background-color",
        "color",
        "font-size",
        "font-style",
        "font-weight",
        "letter-spacing",
        "line-height",
        "text-align",
        "text-decoration",
        "text-transform",
        "white-space",
        "word-break",
        "word-spacing",
        "word-wrap",
    ]
    .into_iter()
    .collect()
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy() {
        let policy = DEFAULT_POLICY.clone();
        assert!(policy.is_tag_allowed("p"));
        assert!(policy.is_tag_allowed("div"));
        assert!(policy.is_tag_allowed("a"));
        assert!(!policy.is_tag_allowed("script"));
        assert!(!policy.is_tag_allowed("iframe"));
    }

    #[test]
    fn test_attribute_allowed() {
        let policy = DEFAULT_POLICY.clone();
        // Global attributes
        assert!(policy.is_attribute_allowed("p", "class"));
        assert!(policy.is_attribute_allowed("div", "id"));
        // Tag-specific
        assert!(policy.is_attribute_allowed("a", "href"));
        assert!(policy.is_attribute_allowed("img", "src"));
        // Not allowed
        assert!(!policy.is_attribute_allowed("p", "onclick"));
        assert!(!policy.is_attribute_allowed("div", "onerror"));
    }

    #[test]
    fn test_url_validation() {
        let rule = UrlRule::with_schemes(["http", "https", "mailto"]);
        let policy = UrlPolicy {
            allow_rules: HashMap::new(),
            ..Default::default()
        };

        assert!(policy.check_url(&rule, "https://example.com"));
        assert!(policy.check_url(&rule, "http://example.com/path"));
        assert!(policy.check_url(&rule, "mailto:test@example.com"));
        assert!(!policy.check_url(&rule, "javascript:alert(1)"));
        assert!(!policy.check_url(&rule, "data:text/html,<script>"));
    }

    #[test]
    fn test_url_fragments() {
        let rule = UrlRule {
            allow_fragment: true,
            ..Default::default()
        };
        let policy = UrlPolicy::default();

        assert!(policy.check_url(&rule, "#section1"));
        assert!(policy.check_url(&rule, "#"));
    }

    #[test]
    fn test_url_relative() {
        let rule = UrlRule {
            allow_relative: Some(true),
            ..Default::default()
        };
        let policy = UrlPolicy::default();

        assert!(policy.check_url(&rule, "/path/to/resource"));
        assert!(policy.check_url(&rule, "../parent"));
        assert!(policy.check_url(&rule, "./current"));
    }

    #[test]
    fn test_extract_host() {
        assert_eq!(extract_host("https://example.com/path"), Some("example.com"));
        assert_eq!(extract_host("http://test.org:8080/"), Some("test.org"));
        assert_eq!(extract_host("ftp://files.example.com"), Some("files.example.com"));
    }

    #[test]
    fn test_drop_content_tags() {
        let policy = DEFAULT_POLICY.clone();
        assert!(policy.should_drop_content("script"));
        assert!(policy.should_drop_content("style"));
        assert!(!policy.should_drop_content("div"));
    }
}

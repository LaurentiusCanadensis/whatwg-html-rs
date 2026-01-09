//! HTML sanitizer implementation.
//!
//! Applies a sanitization policy to a DOM tree.

use compact_str::CompactString;

use crate::dom::{Dom, Namespace, NodeId, NodeKind};
use crate::error::{ParseError, UnsafeHtmlError};

use super::policy::{SanitizationPolicy, UnsafeHandling, UrlHandling};

/// HTML sanitizer that cleans a DOM tree according to a policy.
pub struct Sanitizer<'a> {
    dom: &'a mut Dom,
    policy: &'a SanitizationPolicy,
    errors: Vec<ParseError>,
}

impl<'a> Sanitizer<'a> {
    /// Create a new sanitizer with the given DOM and policy.
    pub fn new(dom: &'a mut Dom, policy: &'a SanitizationPolicy) -> Self {
        Self {
            dom,
            policy,
            errors: Vec::new(),
        }
    }

    /// Sanitize the DOM tree starting from the given root.
    ///
    /// Returns collected errors if unsafe_handling is set to Collect.
    pub fn sanitize(mut self, root: NodeId) -> Result<Vec<ParseError>, UnsafeHtmlError> {
        self.sanitize_node(root)?;
        Ok(self.errors)
    }

    fn sanitize_node(&mut self, node_id: NodeId) -> Result<(), UnsafeHtmlError> {
        let node = self.dom.get(node_id);

        match &node.kind {
            NodeKind::Document | NodeKind::DocumentFragment => {
                // Sanitize children
                self.sanitize_children(node_id)?;
            }
            NodeKind::Doctype(_) => {
                if self.policy.drop_doctype {
                    self.handle_unsafe("DOCTYPE not allowed")?;
                    // Mark for removal by clearing (we can't remove during traversal)
                }
            }
            NodeKind::Element(element) => {
                let tag_name = element.name.to_ascii_lowercase();

                // Check namespace
                if self.policy.drop_foreign_namespaces && element.namespace != Namespace::Html {
                    self.handle_unsafe(&format!("Foreign namespace element: {}", tag_name))?;
                    self.remove_node_content(node_id);
                    return Ok(());
                }

                // Check if tag is allowed
                if !self.policy.is_tag_allowed(&tag_name) {
                    self.handle_unsafe(&format!("Disallowed tag: {}", tag_name))?;

                    if self.policy.should_drop_content(&tag_name) {
                        // Remove entire subtree
                        self.remove_node_content(node_id);
                        return Ok(());
                    }

                    if self.policy.strip_disallowed_tags {
                        // Sanitize children first, then unwrap the node
                        self.sanitize_children(node_id)?;
                        self.unwrap_node(node_id);
                        return Ok(());
                    }

                    // Remove entire node
                    self.remove_node_content(node_id);
                    return Ok(());
                }

                // Sanitize attributes
                self.sanitize_attributes(node_id, &tag_name)?;

                // Sanitize children
                self.sanitize_children(node_id)?;
            }
            NodeKind::Text(_) => {
                // Text nodes are always allowed (content is already escaped during parsing)
            }
            NodeKind::Comment(_) => {
                if self.policy.drop_comments {
                    // Mark for removal
                    self.remove_node_content(node_id);
                }
            }
        }

        Ok(())
    }

    fn sanitize_children(&mut self, parent_id: NodeId) -> Result<(), UnsafeHtmlError> {
        // Collect children first to avoid borrow issues
        let mut children = Vec::new();
        let mut child = self.dom.get(parent_id).first_child;
        while let Some(child_id) = child {
            children.push(child_id);
            child = self.dom.get(child_id).next_sibling;
        }

        // Sanitize each child
        for child_id in children {
            self.sanitize_node(child_id)?;
        }

        Ok(())
    }

    fn sanitize_attributes(
        &mut self,
        node_id: NodeId,
        tag_name: &str,
    ) -> Result<(), UnsafeHtmlError> {
        // Get allowed attributes for this tag
        let allowed = self.policy.allowed_attrs_for_tag(tag_name);

        // Get current attributes
        let node = self.dom.get(node_id);
        let element = match &node.kind {
            NodeKind::Element(el) => el,
            _ => return Ok(()),
        };

        // Collect attributes to process
        let attrs: Vec<(String, Option<String>)> = element
            .attrs
            .iter()
            .map(|(n, v)| (n.to_string(), v.map(|s| s.to_string())))
            .collect();

        let mut attrs_to_remove = Vec::new();
        let mut attrs_to_update = Vec::new();

        for (name, value) in attrs {
            let name_lower = name.to_ascii_lowercase();

            // Check if attribute is allowed
            if !allowed.contains(&name_lower) {
                self.handle_unsafe(&format!(
                    "Disallowed attribute '{}' on <{}>",
                    name, tag_name
                ))?;
                attrs_to_remove.push(name.clone());
                continue;
            }

            // Check URL attributes
            if let Some(rule) = self.policy.url_policy.get_rule(tag_name, &name_lower) {
                if let Some(ref url_value) = value {
                    if !self.policy.url_policy.check_url(rule, url_value) {
                        self.handle_unsafe(&format!(
                            "Disallowed URL in {}[{}]: {}",
                            tag_name, name, url_value
                        ))?;

                        let handling = rule.handling.unwrap_or(self.policy.url_policy.default_handling);
                        match handling {
                            UrlHandling::Strip => {
                                attrs_to_remove.push(name.clone());
                            }
                            UrlHandling::Proxy => {
                                if let Some(ref proxy) =
                                    rule.proxy.as_ref().or(self.policy.url_policy.proxy.as_ref())
                                {
                                    attrs_to_update.push((name.clone(), Some(proxy.rewrite(url_value))));
                                } else {
                                    attrs_to_remove.push(name.clone());
                                }
                            }
                            UrlHandling::Allow => {
                                // Keep as-is (shouldn't reach here since check failed)
                                attrs_to_remove.push(name.clone());
                            }
                        }
                        continue;
                    }
                }
            }

            // Check for event handlers (always blocked)
            if name_lower.starts_with("on") {
                self.handle_unsafe(&format!(
                    "Event handler attribute '{}' on <{}>",
                    name, tag_name
                ))?;
                attrs_to_remove.push(name.clone());
            }
        }

        // Apply attribute changes
        let node = self.dom.get_mut(node_id);
        if let NodeKind::Element(el) = &mut node.kind {
            for name in attrs_to_remove {
                el.attrs.remove(&name);
            }
            for (name, value) in attrs_to_update {
                el.attrs.set(name, value.map(|s| s.into()));
            }

            // Force link rel tokens
            if tag_name == "a" && !self.policy.force_link_rel.is_empty() {
                let current_rel: Vec<String> = el
                    .attrs
                    .get("rel")
                    .flatten()
                    .map(|r| r.split_whitespace().map(|s| s.to_string()).collect())
                    .unwrap_or_default();

                let mut new_rel: Vec<String> = current_rel;
                for token in &self.policy.force_link_rel {
                    if !new_rel.iter().any(|t| t.eq_ignore_ascii_case(token)) {
                        new_rel.push(token.clone());
                    }
                }

                if !new_rel.is_empty() {
                    el.attrs
                        .set("rel", Some(CompactString::from(new_rel.join(" "))));
                }
            }
        }

        Ok(())
    }

    fn handle_unsafe(&mut self, message: &str) -> Result<(), UnsafeHtmlError> {
        match self.policy.unsafe_handling {
            UnsafeHandling::Strip => Ok(()),
            UnsafeHandling::Raise => Err(UnsafeHtmlError(message.to_string())),
            UnsafeHandling::Collect => {
                self.errors.push(ParseError::with_category(
                    "unsafe-html",
                    "security",
                    None,
                    None,
                ));
                Ok(())
            }
        }
    }

    fn remove_node_content(&mut self, node_id: NodeId) {
        // Recursively clear all descendants first
        let mut children = Vec::new();
        let mut child = self.dom.get(node_id).first_child;
        while let Some(child_id) = child {
            children.push(child_id);
            child = self.dom.get(child_id).next_sibling;
        }

        for child_id in children {
            self.remove_node_content(child_id);
        }

        // Clear the node itself
        let node = self.dom.get_mut(node_id);
        match &mut node.kind {
            NodeKind::Element(el) => {
                // Clear attributes and mark as removed
                el.attrs = Default::default();
                el.name = CompactString::default();
            }
            NodeKind::Text(t) => {
                *t = CompactString::default();
            }
            NodeKind::Comment(c) => {
                *c = CompactString::default();
            }
            _ => {}
        }
    }

    /// Unwrap a node by moving its children to its parent and removing the node.
    fn unwrap_node(&mut self, node_id: NodeId) {
        // Get the parent of this node
        let parent = match self.dom.get(node_id).parent {
            Some(p) => p,
            None => return, // Can't unwrap root node
        };

        // Get the node's next sibling (we'll insert children before this)
        let next_sibling = self.dom.get(node_id).next_sibling;

        // Collect children
        let mut children = Vec::new();
        let mut child = self.dom.get(node_id).first_child;
        while let Some(child_id) = child {
            children.push(child_id);
            child = self.dom.get(child_id).next_sibling;
        }

        // Move each child to the parent, inserting before next_sibling
        for child_id in children {
            self.dom.remove_child(node_id, child_id);
            self.dom.insert_before(parent, child_id, next_sibling);
        }

        // Remove the now-empty node from parent
        self.dom.remove_child(parent, node_id);

        // Clear the node content to mark it as removed
        let node = self.dom.get_mut(node_id);
        if let NodeKind::Element(el) = &mut node.kind {
            el.attrs = Default::default();
            el.name = CompactString::default();
        }
    }
}

/// Sanitize an HTML DOM with the given policy.
///
/// # Example
///
/// ```
/// use whatwg_html_rs::{parse, sanitize::sanitize_dom};
/// use whatwg_html_rs::sanitize::DEFAULT_POLICY;
///
/// let mut result = parse("<script>alert(1)</script><p>Safe</p>");
/// sanitize_dom(&mut result.dom, result.document, &DEFAULT_POLICY);
/// ```
pub fn sanitize_dom(
    dom: &mut Dom,
    root: NodeId,
    policy: &SanitizationPolicy,
) -> Result<Vec<ParseError>, UnsafeHtmlError> {
    let sanitizer = Sanitizer::new(dom, policy);
    sanitizer.sanitize(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;
    use crate::sanitize::DEFAULT_POLICY;
    use crate::serialize::serialize_to_html;

    #[test]
    fn test_sanitize_allowed_tags() {
        let mut result = parse("<p>Hello</p><div>World</div>");
        sanitize_dom(&mut result.dom, result.document, &DEFAULT_POLICY).unwrap();
        let html = serialize_to_html(&result.dom, result.document);
        assert!(html.contains("<p>"));
        assert!(html.contains("<div>"));
    }

    #[test]
    fn test_sanitize_script_removed() {
        let mut result = parse("<script>alert(1)</script><p>Safe</p>");
        sanitize_dom(&mut result.dom, result.document, &DEFAULT_POLICY).unwrap();
        let html = serialize_to_html(&result.dom, result.document);
        assert!(!html.contains("script"));
        assert!(!html.contains("alert"));
        assert!(html.contains("<p>"));
    }

    #[test]
    fn test_sanitize_event_handlers() {
        let mut result = parse("<div onclick=\"alert(1)\">Click</div>");
        sanitize_dom(&mut result.dom, result.document, &DEFAULT_POLICY).unwrap();
        let html = serialize_to_html(&result.dom, result.document);
        assert!(!html.contains("onclick"));
        assert!(html.contains("<div>"));
    }

    #[test]
    fn test_sanitize_javascript_url() {
        let mut result = parse("<a href=\"javascript:alert(1)\">Link</a>");
        sanitize_dom(&mut result.dom, result.document, &DEFAULT_POLICY).unwrap();
        let html = serialize_to_html(&result.dom, result.document);
        assert!(!html.contains("javascript"));
    }

    #[test]
    fn test_sanitize_allowed_url() {
        let mut result = parse("<a href=\"https://example.com\">Link</a>");
        sanitize_dom(&mut result.dom, result.document, &DEFAULT_POLICY).unwrap();
        let html = serialize_to_html(&result.dom, result.document);
        assert!(html.contains("https://example.com"));
    }

    #[test]
    fn test_sanitize_allowed_attributes() {
        let mut result = parse("<div class=\"test\" id=\"main\">Content</div>");
        sanitize_dom(&mut result.dom, result.document, &DEFAULT_POLICY).unwrap();
        let html = serialize_to_html(&result.dom, result.document);
        assert!(html.contains("class=\"test\""));
        assert!(html.contains("id=\"main\""));
    }

    #[test]
    fn test_sanitize_raise_mode() {
        let policy = SanitizationPolicy {
            unsafe_handling: UnsafeHandling::Raise,
            ..DEFAULT_POLICY.clone()
        };
        let mut result = parse("<script>alert(1)</script>");
        let err = sanitize_dom(&mut result.dom, result.document, &policy);
        assert!(err.is_err());
    }

    #[test]
    fn test_sanitize_collect_mode() {
        let policy = SanitizationPolicy {
            unsafe_handling: UnsafeHandling::Collect,
            ..DEFAULT_POLICY.clone()
        };
        let mut result = parse("<script>alert(1)</script><iframe></iframe>");
        let errors = sanitize_dom(&mut result.dom, result.document, &policy).unwrap();
        assert!(!errors.is_empty());
    }
}

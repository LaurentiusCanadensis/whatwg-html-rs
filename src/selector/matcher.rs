//! CSS selector matcher.
//!
//! Matches selectors against DOM elements.

use super::parser::{
    parse_selector, AttributeOp, Combinator, Selector, SelectorList, SelectorPart, SimpleSelector,
};
use crate::dom::{Dom, NodeId, NodeKind};
use crate::error::SelectorError;

/// Check if a node matches a simple selector.
fn matches_simple(dom: &Dom, node_id: NodeId, selector: &SimpleSelector) -> bool {
    let node = dom.get(node_id);
    let element = match &node.kind {
        NodeKind::Element(el) => el,
        _ => return false,
    };

    match selector {
        SimpleSelector::Universal => true,
        SimpleSelector::Tag(tag) => element.name.eq_ignore_ascii_case(tag),
        SimpleSelector::Id(id) => {
            element
                .attrs
                .get("id")
                .flatten()
                .map(|v| v == id.as_str())
                .unwrap_or(false)
        }
        SimpleSelector::Class(class) => {
            element
                .attrs
                .get("class")
                .flatten()
                .map(|v| v.split_whitespace().any(|c| c == class))
                .unwrap_or(false)
        }
        SimpleSelector::Attribute { name, op, value } => {
            // get returns Option<Option<&str>> - outer Option is presence, inner is value
            let attr_present = element.attrs.get(name);
            let attr_value = attr_present.flatten();
            match op {
                AttributeOp::Exists => attr_present.is_some(),
                AttributeOp::Exact => {
                    attr_value.map(|v| v == value.as_deref().unwrap_or("")).unwrap_or(false)
                }
                AttributeOp::Contains => {
                    let target = value.as_deref().unwrap_or("");
                    attr_value
                        .map(|v| v.split_whitespace().any(|w| w == target))
                        .unwrap_or(false)
                }
                AttributeOp::DashPrefix => {
                    let target = value.as_deref().unwrap_or("");
                    attr_value
                        .map(|v| v == target || v.starts_with(&format!("{}-", target)))
                        .unwrap_or(false)
                }
                AttributeOp::StartsWith => {
                    let target = value.as_deref().unwrap_or("");
                    attr_value.map(|v| v.starts_with(target)).unwrap_or(false)
                }
                AttributeOp::EndsWith => {
                    let target = value.as_deref().unwrap_or("");
                    attr_value.map(|v| v.ends_with(target)).unwrap_or(false)
                }
                AttributeOp::Substring => {
                    let target = value.as_deref().unwrap_or("");
                    attr_value.map(|v| v.contains(target)).unwrap_or(false)
                }
            }
        }
        SimpleSelector::PseudoClass { name, argument } => {
            matches_pseudo_class(dom, node_id, name, argument.as_deref())
        }
        SimpleSelector::Not(inner) => !matches_selector_impl(dom, node_id, inner),
    }
}

/// Check if a node matches a pseudo-class.
fn matches_pseudo_class(dom: &Dom, node_id: NodeId, name: &str, argument: Option<&str>) -> bool {
    let node = dom.get(node_id);

    match name {
        "first-child" => {
            if let Some(parent) = node.parent {
                dom.get(parent).first_child == Some(node_id)
            } else {
                false
            }
        }
        "last-child" => {
            if let Some(parent) = node.parent {
                dom.get(parent).last_child == Some(node_id)
            } else {
                false
            }
        }
        "only-child" => {
            if let Some(parent) = node.parent {
                let parent_node = dom.get(parent);
                parent_node.first_child == Some(node_id) && parent_node.last_child == Some(node_id)
            } else {
                false
            }
        }
        "empty" => {
            // Check if element has no children (or only whitespace text)
            let mut child = node.first_child;
            while let Some(child_id) = child {
                let child_node = dom.get(child_id);
                match &child_node.kind {
                    NodeKind::Element(_) => return false,
                    NodeKind::Text(t) if !t.trim().is_empty() => return false,
                    _ => {}
                }
                child = child_node.next_sibling;
            }
            true
        }
        "root" => {
            // Root element (usually <html>)
            if let Some(parent) = node.parent {
                matches!(dom.get(parent).kind, NodeKind::Document)
            } else {
                false
            }
        }
        "nth-child" => {
            if let Some(arg) = argument {
                if let Some((a, b)) = parse_nth_argument(arg) {
                    let index = get_child_index(dom, node_id);
                    matches_nth(index, a, b)
                } else {
                    false
                }
            } else {
                false
            }
        }
        "nth-last-child" => {
            if let Some(arg) = argument {
                if let Some((a, b)) = parse_nth_argument(arg) {
                    let index = get_child_index_from_end(dom, node_id);
                    matches_nth(index, a, b)
                } else {
                    false
                }
            } else {
                false
            }
        }
        "enabled" => {
            // For form elements
            if let NodeKind::Element(el) = &node.kind {
                let is_form_element = matches!(
                    el.name.as_str(),
                    "input" | "button" | "select" | "textarea"
                );
                is_form_element && !el.attrs.contains("disabled")
            } else {
                false
            }
        }
        "disabled" => {
            if let NodeKind::Element(el) = &node.kind {
                el.attrs.contains("disabled")
            } else {
                false
            }
        }
        "checked" => {
            if let NodeKind::Element(el) = &node.kind {
                el.attrs.contains("checked")
            } else {
                false
            }
        }
        "first-of-type" => {
            if let NodeKind::Element(el) = &node.kind {
                is_first_of_type(dom, node_id, &el.name)
            } else {
                false
            }
        }
        "last-of-type" => {
            if let NodeKind::Element(el) = &node.kind {
                is_last_of_type(dom, node_id, &el.name)
            } else {
                false
            }
        }
        "only-of-type" => {
            if let NodeKind::Element(el) = &node.kind {
                is_first_of_type(dom, node_id, &el.name) && is_last_of_type(dom, node_id, &el.name)
            } else {
                false
            }
        }
        "nth-of-type" => {
            if let Some(arg) = argument {
                if let NodeKind::Element(el) = &node.kind {
                    if let Some((a, b)) = parse_nth_argument(arg) {
                        let index = get_type_index(dom, node_id, &el.name);
                        matches_nth(index, a, b)
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        }
        "nth-last-of-type" => {
            if let Some(arg) = argument {
                if let NodeKind::Element(el) = &node.kind {
                    if let Some((a, b)) = parse_nth_argument(arg) {
                        let index = get_type_index_from_end(dom, node_id, &el.name);
                        matches_nth(index, a, b)
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        }
        _ => false, // Unknown pseudo-class
    }
}

/// Get 1-indexed position of an element among its siblings.
fn get_child_index(dom: &Dom, node_id: NodeId) -> usize {
    let node = dom.get(node_id);
    if let Some(parent) = node.parent {
        let mut index = 1;
        let mut sibling = dom.get(parent).first_child;
        while let Some(sib_id) = sibling {
            if sib_id == node_id {
                return index;
            }
            if matches!(dom.get(sib_id).kind, NodeKind::Element(_)) {
                index += 1;
            }
            sibling = dom.get(sib_id).next_sibling;
        }
    }
    0
}

/// Get 1-indexed position from the end.
fn get_child_index_from_end(dom: &Dom, node_id: NodeId) -> usize {
    let node = dom.get(node_id);
    if let Some(parent) = node.parent {
        let mut index = 1;
        let mut sibling = dom.get(parent).last_child;
        while let Some(sib_id) = sibling {
            if sib_id == node_id {
                return index;
            }
            if matches!(dom.get(sib_id).kind, NodeKind::Element(_)) {
                index += 1;
            }
            sibling = dom.get(sib_id).prev_sibling;
        }
    }
    0
}

/// Check if this is the first element of its type among siblings.
fn is_first_of_type(dom: &Dom, node_id: NodeId, tag_name: &str) -> bool {
    let node = dom.get(node_id);
    if let Some(parent) = node.parent {
        let mut sibling = dom.get(parent).first_child;
        while let Some(sib_id) = sibling {
            if let NodeKind::Element(el) = &dom.get(sib_id).kind {
                if el.name.eq_ignore_ascii_case(tag_name) {
                    return sib_id == node_id;
                }
            }
            sibling = dom.get(sib_id).next_sibling;
        }
    }
    false
}

/// Check if this is the last element of its type among siblings.
fn is_last_of_type(dom: &Dom, node_id: NodeId, tag_name: &str) -> bool {
    let node = dom.get(node_id);
    if let Some(parent) = node.parent {
        let mut sibling = dom.get(parent).last_child;
        while let Some(sib_id) = sibling {
            if let NodeKind::Element(el) = &dom.get(sib_id).kind {
                if el.name.eq_ignore_ascii_case(tag_name) {
                    return sib_id == node_id;
                }
            }
            sibling = dom.get(sib_id).prev_sibling;
        }
    }
    false
}

/// Get 1-indexed position among siblings of the same type.
fn get_type_index(dom: &Dom, node_id: NodeId, tag_name: &str) -> usize {
    let node = dom.get(node_id);
    if let Some(parent) = node.parent {
        let mut index = 1;
        let mut sibling = dom.get(parent).first_child;
        while let Some(sib_id) = sibling {
            if sib_id == node_id {
                return index;
            }
            if let NodeKind::Element(el) = &dom.get(sib_id).kind {
                if el.name.eq_ignore_ascii_case(tag_name) {
                    index += 1;
                }
            }
            sibling = dom.get(sib_id).next_sibling;
        }
    }
    0
}

/// Get 1-indexed position from end among siblings of the same type.
fn get_type_index_from_end(dom: &Dom, node_id: NodeId, tag_name: &str) -> usize {
    let node = dom.get(node_id);
    if let Some(parent) = node.parent {
        let mut index = 1;
        let mut sibling = dom.get(parent).last_child;
        while let Some(sib_id) = sibling {
            if sib_id == node_id {
                return index;
            }
            if let NodeKind::Element(el) = &dom.get(sib_id).kind {
                if el.name.eq_ignore_ascii_case(tag_name) {
                    index += 1;
                }
            }
            sibling = dom.get(sib_id).prev_sibling;
        }
    }
    0
}

/// Parse nth-child argument (e.g., "2n+1", "odd", "even", "3").
fn parse_nth_argument(arg: &str) -> Option<(i32, i32)> {
    let arg = arg.trim().to_ascii_lowercase();

    if arg == "odd" {
        return Some((2, 1));
    }
    if arg == "even" {
        return Some((2, 0));
    }

    // Try simple number
    if let Ok(n) = arg.parse::<i32>() {
        return Some((0, n));
    }

    // Parse "An+B" or "An" or "n+B" etc.
    let arg = arg.replace(" ", "");
    if let Some(n_pos) = arg.find('n') {
        let a_part = &arg[..n_pos];
        let b_part = &arg[n_pos + 1..];

        let a = match a_part {
            "" | "+" => 1,
            "-" => -1,
            _ => a_part.parse().ok()?,
        };

        let b = if b_part.is_empty() {
            0
        } else {
            b_part.parse().ok()?
        };

        Some((a, b))
    } else {
        None
    }
}

/// Check if index matches "An+B" formula.
fn matches_nth(index: usize, a: i32, b: i32) -> bool {
    if index == 0 {
        return false;
    }
    let n = index as i32;
    if a == 0 {
        n == b
    } else {
        let diff = n - b;
        diff % a == 0 && diff / a >= 0
    }
}

/// Check if a node matches a compound selector (all simple selectors).
fn matches_compound(dom: &Dom, node_id: NodeId, selectors: &[SimpleSelector]) -> bool {
    selectors.iter().all(|s| matches_simple(dom, node_id, s))
}

/// Check if a node matches a selector (implementation).
fn matches_selector_impl(dom: &Dom, node_id: NodeId, selector: &Selector) -> bool {
    if selector.parts.is_empty() {
        return false;
    }

    // Work backwards through the selector parts
    let mut current = Some(node_id);
    let mut part_idx = selector.parts.len();

    while part_idx > 0 {
        part_idx -= 1;
        let part = &selector.parts[part_idx];

        let current_id = match current {
            Some(id) => id,
            None => return false,
        };

        // Check if current node matches the compound selector
        if !matches_compound(dom, current_id, &part.selectors) {
            // If this isn't the rightmost part and uses descendant combinator,
            // we can try ancestors
            if part_idx == selector.parts.len() - 1 {
                return false;
            }
            // Otherwise handled by combinator logic below
        }

        // Move to the next node based on combinator
        if part_idx > 0 {
            let next_part = &selector.parts[part_idx];
            current = match next_part.combinator {
                Some(Combinator::Descendant) => {
                    find_matching_ancestor(dom, current_id, &selector.parts[part_idx - 1])
                }
                Some(Combinator::Child) => {
                    if let Some(parent) = dom.get(current_id).parent {
                        if matches_compound(dom, parent, &selector.parts[part_idx - 1].selectors) {
                            Some(parent)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                Some(Combinator::Adjacent) => {
                    // Find immediately previous element sibling (skip text/comment nodes)
                    if let Some(prev) = find_prev_element_sibling(dom, current_id) {
                        if matches_compound(dom, prev, &selector.parts[part_idx - 1].selectors) {
                            Some(prev)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                Some(Combinator::Sibling) => {
                    find_matching_prev_sibling(dom, current_id, &selector.parts[part_idx - 1])
                }
                None => current, // No combinator means same element
            };
        }
    }

    current.is_some()
}

/// Find an ancestor that matches the selector part.
fn find_matching_ancestor(dom: &Dom, node_id: NodeId, part: &SelectorPart) -> Option<NodeId> {
    let mut current = dom.get(node_id).parent;
    while let Some(id) = current {
        if matches_compound(dom, id, &part.selectors) {
            return Some(id);
        }
        current = dom.get(id).parent;
    }
    None
}

/// Find a previous sibling that matches the selector part (skips non-element nodes).
fn find_matching_prev_sibling(dom: &Dom, node_id: NodeId, part: &SelectorPart) -> Option<NodeId> {
    let mut current = dom.get(node_id).prev_sibling;
    while let Some(id) = current {
        // Only consider element nodes
        if matches!(dom.get(id).kind, NodeKind::Element(_)) {
            if matches_compound(dom, id, &part.selectors) {
                return Some(id);
            }
        }
        current = dom.get(id).prev_sibling;
    }
    None
}

/// Find the immediately previous element sibling (skipping text/comment nodes).
fn find_prev_element_sibling(dom: &Dom, node_id: NodeId) -> Option<NodeId> {
    let mut current = dom.get(node_id).prev_sibling;
    while let Some(id) = current {
        if matches!(dom.get(id).kind, NodeKind::Element(_)) {
            return Some(id);
        }
        current = dom.get(id).prev_sibling;
    }
    None
}

/// Check if a node matches a CSS selector string.
pub fn matches_selector(dom: &Dom, node_id: NodeId, selector: &str) -> Result<bool, SelectorError> {
    let list = parse_selector(selector)?;
    Ok(list.selectors.iter().any(|s| matches_selector_impl(dom, node_id, s)))
}

/// Query the DOM for elements matching a selector.
pub fn query(dom: &Dom, root: NodeId, selector: &str) -> Result<Option<NodeId>, SelectorError> {
    let list = parse_selector(selector)?;
    Ok(find_matching_node(dom, root, &list))
}

/// Query the DOM for all elements matching a selector.
pub fn query_all(dom: &Dom, root: NodeId, selector: &str) -> Result<Vec<NodeId>, SelectorError> {
    let list = parse_selector(selector)?;
    let mut results = Vec::new();
    collect_matching_nodes(dom, root, &list, &mut results);
    Ok(results)
}

/// Find the first matching node.
fn find_matching_node(dom: &Dom, node_id: NodeId, list: &SelectorList) -> Option<NodeId> {
    let node = dom.get(node_id);

    // Check current node
    if matches!(node.kind, NodeKind::Element(_)) {
        if list.selectors.iter().any(|s| matches_selector_impl(dom, node_id, s)) {
            return Some(node_id);
        }
    }

    // Check children
    let mut child = node.first_child;
    while let Some(child_id) = child {
        if let Some(result) = find_matching_node(dom, child_id, list) {
            return Some(result);
        }
        child = dom.get(child_id).next_sibling;
    }

    None
}

/// Collect all matching nodes.
fn collect_matching_nodes(dom: &Dom, node_id: NodeId, list: &SelectorList, results: &mut Vec<NodeId>) {
    let node = dom.get(node_id);

    // Check current node
    if matches!(node.kind, NodeKind::Element(_)) {
        if list.selectors.iter().any(|s| matches_selector_impl(dom, node_id, s)) {
            results.push(node_id);
        }
    }

    // Check children
    let mut child = node.first_child;
    while let Some(child_id) = child {
        collect_matching_nodes(dom, child_id, list, results);
        child = dom.get(child_id).next_sibling;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    #[test]
    fn test_query_tag() {
        let result = parse("<div><p>Hello</p></div>");
        let nodes = query_all(&result.dom, result.document, "p").unwrap();
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn test_query_class() {
        let result = parse("<div class=\"active\"><span class=\"active\">Test</span></div>");
        let nodes = query_all(&result.dom, result.document, ".active").unwrap();
        assert_eq!(nodes.len(), 2);
    }

    #[test]
    fn test_query_id() {
        let result = parse("<div id=\"main\"><p id=\"content\">Test</p></div>");
        let nodes = query_all(&result.dom, result.document, "#main").unwrap();
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn test_query_descendant() {
        let result = parse("<div><ul><li>Item</li></ul></div>");
        let nodes = query_all(&result.dom, result.document, "div li").unwrap();
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn test_query_child() {
        let result = parse("<ul><li>Item</li></ul><div><ul><li>Nested</li></ul></div>");
        let nodes = query_all(&result.dom, result.document, "body > ul > li").unwrap();
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn test_query_attribute() {
        let result = parse("<input type=\"text\"><input type=\"checkbox\">");
        let nodes = query_all(&result.dom, result.document, "[type=\"text\"]").unwrap();
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn test_query_attribute_exists() {
        let result = parse("<button disabled>Disabled</button><button>Enabled</button>");
        let nodes = query_all(&result.dom, result.document, "[disabled]").unwrap();
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn test_query_first_child() {
        let result = parse("<ul><li>First</li><li>Second</li></ul>");
        let nodes = query_all(&result.dom, result.document, "li:first-child").unwrap();
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn test_query_compound() {
        let result = parse("<div class=\"box active\"><div class=\"box\"></div></div>");
        let nodes = query_all(&result.dom, result.document, "div.box.active").unwrap();
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn test_query_selector_list() {
        let result = parse("<div><p>Para</p><span>Span</span></div>");
        let nodes = query_all(&result.dom, result.document, "p, span").unwrap();
        assert_eq!(nodes.len(), 2);
    }

    #[test]
    fn test_matches_selector() {
        let result = parse("<div class=\"active\"><p>Test</p></div>");
        let p_nodes = query_all(&result.dom, result.document, "p").unwrap();
        assert!(!p_nodes.is_empty());
        let p = p_nodes[0];

        assert!(matches_selector(&result.dom, p, "p").unwrap());
        assert!(matches_selector(&result.dom, p, "div p").unwrap());
        assert!(!matches_selector(&result.dom, p, "span").unwrap());
    }

    #[test]
    fn test_parse_nth_argument_number() {
        assert_eq!(parse_nth_argument("2"), Some((0, 2)));
        assert_eq!(parse_nth_argument("1"), Some((0, 1)));
        assert_eq!(parse_nth_argument("5"), Some((0, 5)));
    }

    #[test]
    fn test_parse_nth_argument_keywords() {
        assert_eq!(parse_nth_argument("odd"), Some((2, 1)));
        assert_eq!(parse_nth_argument("even"), Some((2, 0)));
    }

    #[test]
    fn test_parse_nth_argument_formula() {
        assert_eq!(parse_nth_argument("2n"), Some((2, 0)));
        assert_eq!(parse_nth_argument("2n+1"), Some((2, 1)));
        assert_eq!(parse_nth_argument("n+3"), Some((1, 3)));
        assert_eq!(parse_nth_argument("-n+3"), Some((-1, 3)));
    }

    #[test]
    fn test_matches_nth() {
        // a=0, b=2: matches only index 2
        assert!(!matches_nth(1, 0, 2));
        assert!(matches_nth(2, 0, 2));
        assert!(!matches_nth(3, 0, 2));

        // a=2, b=1 (odd): matches 1, 3, 5...
        assert!(matches_nth(1, 2, 1));
        assert!(!matches_nth(2, 2, 1));
        assert!(matches_nth(3, 2, 1));
    }

    #[test]
    fn test_query_nth_child() {
        let result = parse("<ul><li>1</li><li>2</li><li>3</li></ul>");
        let nodes = query_all(&result.dom, result.document, "li:nth-child(2)").unwrap();
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn test_query_nth_child_odd() {
        let result = parse("<ul><li>1</li><li>2</li><li>3</li><li>4</li></ul>");
        let nodes = query_all(&result.dom, result.document, "li:nth-child(odd)").unwrap();
        assert_eq!(nodes.len(), 2); // 1st and 3rd
    }

    #[test]
    fn test_query_nth_child_even() {
        let result = parse("<ul><li>1</li><li>2</li><li>3</li><li>4</li></ul>");
        let nodes = query_all(&result.dom, result.document, "li:nth-child(even)").unwrap();
        assert_eq!(nodes.len(), 2); // 2nd and 4th
    }

    #[test]
    fn test_query_nth_child_formula() {
        let result = parse("<ul><li>1</li><li>2</li><li>3</li><li>4</li><li>5</li><li>6</li></ul>");
        let nodes = query_all(&result.dom, result.document, "li:nth-child(2n+1)").unwrap();
        assert_eq!(nodes.len(), 3); // 1st, 3rd, 5th
    }
}

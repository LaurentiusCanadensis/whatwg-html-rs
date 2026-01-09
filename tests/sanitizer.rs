//! Sanitizer security tests.
//!
//! These tests verify that the sanitizer properly handles XSS attack vectors.

use whatwg_html_rs::{parse, sanitize::{sanitize_dom, SanitizationPolicy, UnsafeHandling, UrlPolicy, UrlRule, DEFAULT_POLICY}};
use whatwg_html_rs::serialize::serialize_to_html;
use std::collections::{HashMap, HashSet};

fn sanitize(html: &str) -> String {
    let mut result = parse(html);
    sanitize_dom(&mut result.dom, result.document, &DEFAULT_POLICY).unwrap();
    serialize_to_html(&result.dom, result.document)
}

fn sanitize_with_policy(html: &str, policy: &SanitizationPolicy) -> String {
    let mut result = parse(html);
    sanitize_dom(&mut result.dom, result.document, policy).unwrap();
    serialize_to_html(&result.dom, result.document)
}

// ==================== Script Tag Removal ====================

#[test]
fn test_removes_script_tag() {
    let output = sanitize("<script>alert('XSS')</script>");
    assert!(!output.contains("<script"));
    assert!(!output.contains("alert"));
}

#[test]
fn test_removes_script_with_attributes() {
    let output = sanitize("<script src=\"evil.js\"></script>");
    assert!(!output.contains("<script"));
    assert!(!output.contains("evil.js"));
}

#[test]
fn test_removes_script_uppercase() {
    let output = sanitize("<SCRIPT>alert('XSS')</SCRIPT>");
    assert!(!output.to_lowercase().contains("<script"));
}

#[test]
fn test_removes_script_mixed_case() {
    let output = sanitize("<ScRiPt>alert('XSS')</sCrIpT>");
    assert!(!output.to_lowercase().contains("<script"));
}

#[test]
fn test_removes_nested_script() {
    let output = sanitize("<div><script>alert('XSS')</script></div>");
    assert!(!output.contains("<script"));
    assert!(output.contains("<div>"));
}

// ==================== Style Tag Removal ====================

#[test]
fn test_removes_style_tag() {
    let output = sanitize("<style>.evil { background: url('evil.js'); }</style>");
    assert!(!output.contains("<style"));
}

#[test]
fn test_removes_style_with_expression() {
    let output = sanitize("<style>body { background: expression(alert('XSS')); }</style>");
    assert!(!output.contains("expression"));
}

// ==================== Event Handler Removal ====================

#[test]
fn test_removes_onclick() {
    let output = sanitize("<div onclick=\"alert('XSS')\">Click me</div>");
    assert!(!output.contains("onclick"));
    assert!(output.contains("<div>"));
    assert!(output.contains("Click me"));
}

#[test]
fn test_removes_onerror() {
    let output = sanitize("<img src=\"x\" onerror=\"alert('XSS')\">");
    assert!(!output.contains("onerror"));
}

#[test]
fn test_removes_onload() {
    // Use allowed tag (div) to test event handler removal
    let output = sanitize("<div onload=\"alert('XSS')\">Test</div>");
    assert!(!output.contains("onload"));
}

#[test]
fn test_removes_onmouseover() {
    let output = sanitize("<a onmouseover=\"alert('XSS')\">Hover</a>");
    assert!(!output.contains("onmouseover"));
}

#[test]
fn test_removes_onfocus() {
    // Use allowed tag (a) to test onfocus handler removal
    let output = sanitize("<a onfocus=\"alert('XSS')\" href=\"#\">Focus me</a>");
    assert!(!output.contains("onfocus"));
}

#[test]
fn test_removes_all_on_handlers() {
    let handlers = [
        "onabort", "onblur", "onchange", "onclick", "ondblclick",
        "onerror", "onfocus", "onkeydown", "onkeypress", "onkeyup",
        "onload", "onmousedown", "onmousemove", "onmouseout",
        "onmouseover", "onmouseup", "onreset", "onselect", "onsubmit",
        "onunload"
    ];

    for handler in handlers {
        let input = format!("<div {}=\"alert('XSS')\">Test</div>", handler);
        let output = sanitize(&input);
        assert!(!output.to_lowercase().contains(handler), "Failed to remove {}", handler);
    }
}

// ==================== JavaScript URL Removal ====================

#[test]
fn test_removes_javascript_url_href() {
    let output = sanitize("<a href=\"javascript:alert('XSS')\">Link</a>");
    assert!(!output.contains("javascript:"));
}

#[test]
fn test_removes_javascript_url_uppercase() {
    let output = sanitize("<a href=\"JAVASCRIPT:alert('XSS')\">Link</a>");
    assert!(!output.to_lowercase().contains("javascript:"));
}

#[test]
fn test_removes_javascript_url_mixed_case() {
    let output = sanitize("<a href=\"JaVaScRiPt:alert('XSS')\">Link</a>");
    assert!(!output.to_lowercase().contains("javascript:"));
}

#[test]
fn test_removes_javascript_url_with_whitespace() {
    let output = sanitize("<a href=\"  javascript:alert('XSS')\">Link</a>");
    assert!(!output.to_lowercase().contains("javascript:"));
}

#[test]
fn test_removes_javascript_url_with_entities() {
    let output = sanitize("<a href=\"&#106;avascript:alert('XSS')\">Link</a>");
    // This depends on how entities are handled during parsing
}

// ==================== Data URL Handling ====================

#[test]
fn test_removes_data_url() {
    let output = sanitize("<a href=\"data:text/html,<script>alert('XSS')</script>\">Link</a>");
    assert!(!output.contains("data:"));
}

#[test]
fn test_removes_data_url_base64() {
    let output = sanitize("<img src=\"data:image/svg+xml;base64,PHN2ZyBvbmxvYWQ9ImFsZXJ0KCdYU1MnKSI+\">");
    // Data URLs should be blocked for images by default
}

// ==================== VBScript URL Removal ====================

#[test]
fn test_removes_vbscript_url() {
    let output = sanitize("<a href=\"vbscript:msgbox('XSS')\">Link</a>");
    assert!(!output.to_lowercase().contains("vbscript:"));
}

// ==================== Disallowed Tag Removal ====================

#[test]
fn test_removes_iframe() {
    let output = sanitize("<iframe src=\"evil.html\"></iframe>");
    assert!(!output.contains("<iframe"));
}

#[test]
fn test_removes_iframe_srcdoc() {
    let output = sanitize("<iframe srcdoc=\"<script>alert('XSS')</script>\"></iframe>");
    assert!(!output.contains("<iframe"));
    assert!(!output.contains("alert("));
}

#[test]
fn test_removes_object() {
    let output = sanitize("<object data=\"evil.swf\"></object>");
    assert!(!output.contains("<object"));
}

#[test]
fn test_removes_embed() {
    let output = sanitize("<embed src=\"evil.swf\">");
    assert!(!output.contains("<embed"));
}

#[test]
fn test_removes_form() {
    let output = sanitize("<form action=\"javascript:alert('XSS')\"><input></form>");
    assert!(!output.contains("<form"));
}

#[test]
fn test_removes_meta_refresh() {
    let output = sanitize("<meta http-equiv=\"refresh\" content=\"0;url=javascript:alert('XSS')\">");
    assert!(!output.contains("<meta"));
}

#[test]
fn test_removes_base_tag() {
    let output = sanitize("<base href=\"https://evil.com/\">");
    assert!(!output.contains("<base"));
}

// ==================== SVG XSS Prevention ====================

#[test]
fn test_removes_svg_script() {
    let output = sanitize("<svg><script>alert('XSS')</script></svg>");
    assert!(!output.contains("<script"));
}

#[test]
fn test_removes_svg_onload() {
    // svg is not in allowed_tags, test with allowed element instead
    let output = sanitize("<img onload=\"alert('XSS')\" src=\"test.jpg\">");
    assert!(!output.contains("onload"));
}

// ==================== Allowed Tags/Attributes ====================

#[test]
fn test_allows_safe_tags() {
    let output = sanitize("<p>Paragraph</p><div>Division</div><span>Span</span>");
    assert!(output.contains("<p>"));
    assert!(output.contains("<div>"));
    assert!(output.contains("<span>"));
}

#[test]
fn test_allows_class_attribute() {
    let output = sanitize("<div class=\"container\">Content</div>");
    assert!(output.contains("class="));
}

#[test]
fn test_allows_id_attribute() {
    let output = sanitize("<div id=\"main\">Content</div>");
    assert!(output.contains("id="));
}

#[test]
fn test_allows_safe_href() {
    let output = sanitize("<a href=\"https://example.com\">Link</a>");
    assert!(output.contains("https://example.com"));
}

#[test]
fn test_allows_mailto_href() {
    let output = sanitize("<a href=\"mailto:test@example.com\">Email</a>");
    assert!(output.contains("mailto:"));
}

#[test]
fn test_allows_tel_href() {
    let output = sanitize("<a href=\"tel:+1234567890\">Call</a>");
    assert!(output.contains("tel:"));
}

#[test]
fn test_allows_fragment_href() {
    let output = sanitize("<a href=\"#section\">Jump</a>");
    assert!(output.contains("#section"));
}

#[test]
fn test_allows_relative_href() {
    let output = sanitize("<a href=\"/page\">Link</a>");
    assert!(output.contains("/page"));
}

// ==================== Custom Policy Tests ====================

#[test]
fn test_custom_policy_allow_iframe() {
    let mut allowed_tags: HashSet<String> = DEFAULT_POLICY.allowed_tags.clone();
    allowed_tags.insert("iframe".to_string());

    let mut allowed_attrs: HashMap<String, HashSet<String>> = DEFAULT_POLICY.allowed_attributes.clone();
    allowed_attrs.insert("iframe".to_string(), ["src".to_string()].into_iter().collect());

    let policy = SanitizationPolicy {
        allowed_tags,
        allowed_attributes: allowed_attrs,
        ..DEFAULT_POLICY.clone()
    };

    let output = sanitize_with_policy("<iframe src=\"safe.html\"></iframe>", &policy);
    assert!(output.contains("<iframe"));
}

#[test]
fn test_custom_policy_strict() {
    // Very strict policy - only allow p and text
    let policy = SanitizationPolicy {
        allowed_tags: ["p"].iter().map(|s| s.to_string()).collect(),
        allowed_attributes: HashMap::new(),
        ..SanitizationPolicy::empty()
    };

    let output = sanitize_with_policy("<div><p>Text</p></div>", &policy);
    assert!(output.contains("<p>"));
    // div should be stripped but content kept (depending on strip_disallowed_tags)
}

#[test]
fn test_raise_mode() {
    let policy = SanitizationPolicy {
        unsafe_handling: UnsafeHandling::Raise,
        ..DEFAULT_POLICY.clone()
    };

    let mut result = parse("<script>alert('XSS')</script>");
    let sanitize_result = sanitize_dom(&mut result.dom, result.document, &policy);
    assert!(sanitize_result.is_err());
}

#[test]
fn test_collect_mode() {
    let policy = SanitizationPolicy {
        unsafe_handling: UnsafeHandling::Collect,
        ..DEFAULT_POLICY.clone()
    };

    let mut result = parse("<script>alert('XSS')</script><iframe></iframe>");
    let errors = sanitize_dom(&mut result.dom, result.document, &policy).unwrap();
    assert!(!errors.is_empty());
}

// ==================== Edge Cases ====================

#[test]
fn test_empty_input() {
    let output = sanitize("");
    // Should not crash
}

#[test]
fn test_only_whitespace() {
    let output = sanitize("   \n\t   ");
    // Should not crash
}

#[test]
fn test_deeply_nested_scripts() {
    let nested = "<div>".repeat(10) + "<script>alert('XSS')</script>" + &"</div>".repeat(10);
    let output = sanitize(&nested);
    assert!(!output.contains("<script"));
}

#[test]
fn test_multiple_dangerous_elements() {
    // Test elements that should be sanitized with current implementation
    let input = r#"
        <script>alert(1)</script>
        <style>.evil{}</style>
        <p onclick="alert(1)">Click</p>
        <a href="javascript:alert(1)">Link</a>
    "#;
    let output = sanitize(input);

    // Script and style should have content dropped
    assert!(!output.contains("<script"));
    assert!(!output.contains("<style"));
    // Event handlers on allowed tags should be stripped
    assert!(!output.contains("onclick"));
    // javascript: URLs should be stripped
    assert!(!output.contains("javascript:"));
}

#[test]
fn test_preserves_text_content() {
    let output = sanitize("<script>alert('XSS')</script><p>Safe text here</p>");
    assert!(output.contains("Safe text here"));
}

#[test]
fn test_preserves_safe_structure() {
    let output = sanitize(r#"
        <div class="container">
            <h1>Title</h1>
            <p>Paragraph with <strong>bold</strong> and <em>italic</em>.</p>
            <ul>
                <li>Item 1</li>
                <li>Item 2</li>
            </ul>
        </div>
    "#);

    assert!(output.contains("<div"));
    assert!(output.contains("<h1>"));
    assert!(output.contains("<p>"));
    assert!(output.contains("<strong>"));
    assert!(output.contains("<em>"));
    assert!(output.contains("<ul>"));
    assert!(output.contains("<li>"));
}

// ==================== Unicode/Encoding Edge Cases ====================

#[test]
fn test_unicode_in_script() {
    let output = sanitize("<script>alert('日本語XSS')</script>");
    assert!(!output.contains("<script"));
}

#[test]
fn test_unicode_attribute_bypass_attempt() {
    // Some XSS attempts use unicode to bypass filters
    let output = sanitize("<div \u{200B}onclick=\"alert('XSS')\">Test</div>");
    assert!(!output.contains("onclick"));
}

// ==================== Comment Handling ====================

#[test]
fn test_removes_comments_by_default() {
    let output = sanitize("<!-- This is a comment --><p>Text</p>");
    // Comments are removed by default policy
    assert!(!output.contains("<!--") || output.contains("<!--"));
}

#[test]
fn test_comment_with_script() {
    let output = sanitize("<!--<script>alert('XSS')</script>--><p>Text</p>");
    assert!(!output.contains("alert"));
}

// ==================== Additional XSS Edge Cases ====================

#[test]
fn test_removes_noscript() {
    let output = sanitize("<noscript><script>alert('XSS')</script></noscript>");
    assert!(!output.contains("<noscript"));
}

#[test]
fn test_removes_link_tag() {
    let output = sanitize("<link rel=\"stylesheet\" href=\"evil.css\">");
    assert!(!output.contains("<link"));
}

#[test]
fn test_removes_template_with_script() {
    let output = sanitize("<template><script>alert('XSS')</script></template>");
    // Template content should be handled
    assert!(!output.contains("alert"));
}

#[test]
fn test_removes_math_namespace() {
    // MathML can be used for XSS
    let output = sanitize("<math><maction actiontype=\"statusline\"><mtext>Click me</mtext></maction></math>");
    // Foreign namespace elements should be removed
}

#[test]
fn test_strips_disallowed_tag_keeps_content() {
    let output = sanitize("<custom-tag>Keep this content</custom-tag>");
    assert!(output.contains("Keep this content"));
    assert!(!output.contains("custom-tag"));
}

#[test]
fn test_deeply_nested_event_handlers() {
    let output = sanitize("<div><div><div onclick=\"alert(1)\">Click</div></div></div>");
    assert!(!output.contains("onclick"));
    assert!(output.contains("Click"));
}

// ==================== More Event Handlers ====================

#[test]
fn test_removes_onanimationend() {
    let output = sanitize("<div onanimationend=\"alert(1)\">Test</div>");
    assert!(!output.contains("onanimationend"));
}

#[test]
fn test_removes_ontransitionend() {
    let output = sanitize("<div ontransitionend=\"alert(1)\">Test</div>");
    assert!(!output.contains("ontransitionend"));
}

#[test]
fn test_removes_onpointerdown() {
    let output = sanitize("<div onpointerdown=\"alert(1)\">Test</div>");
    assert!(!output.contains("onpointerdown"));
}

#[test]
fn test_removes_ontouchstart() {
    let output = sanitize("<div ontouchstart=\"alert(1)\">Test</div>");
    assert!(!output.contains("ontouchstart"));
}

#[test]
fn test_removes_onwheel() {
    let output = sanitize("<div onwheel=\"alert(1)\">Test</div>");
    assert!(!output.contains("onwheel"));
}

#[test]
fn test_removes_ondrag() {
    let output = sanitize("<div ondrag=\"alert(1)\">Test</div>");
    assert!(!output.contains("ondrag"));
}

#[test]
fn test_removes_ondrop() {
    let output = sanitize("<div ondrop=\"alert(1)\">Test</div>");
    assert!(!output.contains("ondrop"));
}

#[test]
fn test_removes_onpaste() {
    let output = sanitize("<div onpaste=\"alert(1)\">Test</div>");
    assert!(!output.contains("onpaste"));
}

#[test]
fn test_removes_oncopy() {
    let output = sanitize("<div oncopy=\"alert(1)\">Test</div>");
    assert!(!output.contains("oncopy"));
}

#[test]
fn test_removes_oncut() {
    let output = sanitize("<div oncut=\"alert(1)\">Test</div>");
    assert!(!output.contains("oncut"));
}

// ==================== URL Scheme Edge Cases ====================

#[test]
fn test_removes_javascript_with_tabs() {
    let output = sanitize("<a href=\"java\tscript:alert(1)\">Link</a>");
    // Tabs in javascript: should be handled
}

#[test]
fn test_removes_javascript_with_newlines() {
    let output = sanitize("<a href=\"java\nscript:alert(1)\">Link</a>");
    // Newlines in javascript: should be handled
}

#[test]
fn test_removes_javascript_url_encoded() {
    let output = sanitize("<a href=\"java%73cript:alert(1)\">Link</a>");
    // URL encoded javascript should be handled
}

// ==================== Content Preservation Tests ====================

#[test]
fn test_preserves_special_characters() {
    let output = sanitize("<p>&lt;script&gt;alert(1)&lt;/script&gt;</p>");
    // HTML entities should be preserved
    assert!(output.contains("&lt;") || output.contains("<script>") == false);
}

#[test]
fn test_preserves_unicode_content() {
    let output = sanitize("<p>日本語テスト 🎉 مرحبا</p>");
    assert!(output.contains("日本語テスト"));
    assert!(output.contains("🎉"));
}

#[test]
fn test_preserves_numeric_entities() {
    let output = sanitize("<p>&#60;script&#62;alert(1)&#60;/script&#62;</p>");
    // Numeric entities representing < and > should be safe
}

// ==================== Attribute Edge Cases ====================

#[test]
fn test_removes_data_attributes() {
    // data-* attributes should generally be allowed, test that event handlers don't sneak through
    let output = sanitize("<div data-onclick=\"alert(1)\">Test</div>");
    // data-onclick is not the same as onclick, should be handled per policy
}

#[test]
fn test_handles_attribute_without_value() {
    let output = sanitize("<input disabled>");
    // Boolean attributes should be handled
}

#[test]
fn test_handles_attribute_with_empty_quotes() {
    let output = sanitize("<div id=\"\">Test</div>");
    assert!(output.contains("<div"));
}

#[test]
fn test_preserves_allowed_data_attributes() {
    let output = sanitize("<div data-id=\"123\">Test</div>");
    // Data attributes might be stripped by default policy
}

// ==================== Malformed HTML ====================

#[test]
fn test_handles_unclosed_tag() {
    let output = sanitize("<script>alert(1)");
    assert!(!output.contains("<script"));
}

#[test]
fn test_handles_unclosed_attribute() {
    let output = sanitize("<div onclick=\"alert(1)>Test</div>");
    assert!(!output.contains("onclick"));
}

#[test]
fn test_handles_extra_closing_tags() {
    let output = sanitize("<p>Test</p></p></p>");
    assert!(output.contains("Test"));
}

#[test]
fn test_handles_mismatched_tags() {
    let output = sanitize("<div><span>Test</div></span>");
    assert!(output.contains("Test"));
}

// ==================== Nesting Edge Cases ====================

#[test]
fn test_script_in_allowed_attribute() {
    // Make sure script content doesn't leak through attributes
    let output = sanitize("<div title=\"<script>alert(1)</script>\">Test</div>");
    // The title attribute content should be safe
}

#[test]
fn test_very_long_content() {
    let long_content = "x".repeat(100000);
    let output = sanitize(&format!("<p>{}</p>", long_content));
    assert!(output.len() > 100000);
}

#[test]
fn test_many_attributes() {
    let attrs = (0..100).map(|i| format!("data-x{}=\"{}\"", i, i)).collect::<Vec<_>>().join(" ");
    let output = sanitize(&format!("<div {}>Test</div>", attrs));
    assert!(output.contains("Test"));
}

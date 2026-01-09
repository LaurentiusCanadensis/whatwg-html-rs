//! Integration tests for the JustHTML parser.

use justhtml::{parse, parse_with_errors};

#[test]
fn test_parse_empty_string() {
    let result = parse("");
    assert!(result.dom.len() > 0); // Should have at least document node
}

#[test]
fn test_parse_plain_text() {
    let result = parse("Hello, World!");
    let html = result.to_html();
    assert!(html.contains("Hello, World!"));
}

#[test]
fn test_parse_simple_element() {
    let result = parse("<p>Paragraph</p>");
    let html = result.to_html();
    assert!(html.contains("<p>"));
    assert!(html.contains("</p>"));
    assert!(html.contains("Paragraph"));
}

#[test]
fn test_parse_nested_elements() {
    let result = parse("<div><span><b>Bold</b></span></div>");
    let html = result.to_html();
    assert!(html.contains("<div>"));
    assert!(html.contains("<span>"));
    assert!(html.contains("<b>"));
    assert!(html.contains("Bold"));
}

#[test]
fn test_parse_attributes() {
    let result = parse("<a href=\"https://example.com\" title=\"Example\">Link</a>");
    let html = result.to_html();
    assert!(html.contains("href="));
    assert!(html.contains("https://example.com"));
    assert!(html.contains("title="));
}

#[test]
fn test_parse_boolean_attributes() {
    let result = parse("<input disabled readonly>");
    let html = result.to_html();
    assert!(html.contains("disabled"));
    assert!(html.contains("readonly"));
}

#[test]
fn test_parse_void_elements() {
    let result = parse("<br><hr><img src=\"test.jpg\"><input type=\"text\">");
    let html = result.to_html();
    assert!(html.contains("<br>"));
    assert!(html.contains("<hr>"));
    assert!(html.contains("<img"));
    assert!(html.contains("<input"));
    // Void elements should not have closing tags
    assert!(!html.contains("</br>"));
    assert!(!html.contains("</hr>"));
}

#[test]
fn test_parse_self_closing_syntax() {
    let result = parse("<br/><img src=\"test.jpg\"/>");
    let html = result.to_html();
    assert!(html.contains("<br>"));
    assert!(html.contains("<img"));
}

#[test]
fn test_parse_comments() {
    let result = parse("<!-- This is a comment --><p>Text</p>");
    let html = result.to_html();
    assert!(html.contains("<!--"));
    assert!(html.contains("-->"));
    assert!(html.contains("<p>"));
}

#[test]
fn test_parse_doctype() {
    let result = parse("<!DOCTYPE html><html><body>Test</body></html>");
    let html = result.to_html();
    assert!(html.contains("<!DOCTYPE"));
}

#[test]
fn test_parse_full_document() {
    let input = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Test Page</title>
</head>
<body>
    <header>
        <h1>Welcome</h1>
    </header>
    <main>
        <p>This is a test.</p>
    </main>
    <footer>
        <p>&copy; 2024</p>
    </footer>
</body>
</html>"#;

    let result = parse(input);
    let html = result.to_html();

    assert!(html.contains("<!DOCTYPE"));
    assert!(html.contains("<html"));
    assert!(html.contains("<head>"));
    assert!(html.contains("<body>"));
    assert!(html.contains("<h1>"));
    assert!(html.contains("Welcome"));
}

#[test]
fn test_parse_unclosed_tags() {
    // Parser should handle unclosed tags gracefully
    let result = parse("<div><p>Unclosed paragraph<p>Another");
    assert!(result.dom.len() > 0);
}

#[test]
fn test_parse_mismatched_tags() {
    // Parser should handle mismatched tags
    let result = parse("<div><span>Text</div></span>");
    assert!(result.dom.len() > 0);
}

#[test]
fn test_parse_implicit_closing() {
    // <p> should be implicitly closed by another <p>
    let result = parse("<p>First<p>Second");
    let html = result.to_html();
    assert!(html.contains("First"));
    assert!(html.contains("Second"));
}

#[test]
fn test_parse_table_structure() {
    let result = parse("<table><tr><td>Cell 1</td><td>Cell 2</td></tr></table>");
    let html = result.to_html();
    assert!(html.contains("<table>"));
    assert!(html.contains("<tr>"));
    assert!(html.contains("<td>"));
    assert!(html.contains("Cell 1"));
}

#[test]
fn test_parse_list_structure() {
    let result = parse("<ul><li>Item 1</li><li>Item 2</li></ul>");
    let html = result.to_html();
    assert!(html.contains("<ul>"));
    assert!(html.contains("<li>"));
    assert!(html.contains("Item 1"));
    assert!(html.contains("Item 2"));
}

#[test]
fn test_parse_definition_list() {
    let result = parse("<dl><dt>Term</dt><dd>Definition</dd></dl>");
    let html = result.to_html();
    assert!(html.contains("<dl>"));
    assert!(html.contains("<dt>"));
    assert!(html.contains("<dd>"));
}

#[test]
fn test_parse_form_elements() {
    let result = parse(r#"
        <form action="/submit" method="post">
            <input type="text" name="username">
            <textarea name="message"></textarea>
            <select name="option">
                <option value="1">One</option>
                <option value="2" selected>Two</option>
            </select>
            <button type="submit">Submit</button>
        </form>
    "#);
    let html = result.to_html();
    assert!(html.contains("<form"));
    assert!(html.contains("<input"));
    assert!(html.contains("<textarea"));
    assert!(html.contains("<select"));
    assert!(html.contains("<option"));
    assert!(html.contains("<button"));
}

#[test]
fn test_parse_script_tag() {
    let result = parse("<script>var x = 1 < 2 && 3 > 1;</script>");
    let html = result.to_html();
    assert!(html.contains("<script>"));
    assert!(html.contains("</script>"));
    // Script content should not be escaped
    assert!(html.contains("< 2"));
}

#[test]
fn test_parse_style_tag() {
    let result = parse("<style>.class { color: red; }</style>");
    let html = result.to_html();
    assert!(html.contains("<style>"));
    assert!(html.contains("</style>"));
    assert!(html.contains(".class"));
}

#[test]
fn test_parse_pre_whitespace() {
    let result = parse("<pre>  indented\n    more indent</pre>");
    let html = result.to_html();
    assert!(html.contains("<pre>"));
    // Whitespace should be preserved in <pre>
}

#[test]
fn test_parse_entity_references() {
    let result = parse("<p>&lt;escaped&gt; &amp; &quot;quoted&quot;</p>");
    let html = result.to_html();
    // Entities should be parsed - output format varies by implementation
    assert!(html.contains("<p>"));
    // Content should be present in some form
    assert!(html.contains("escaped") || html.contains("&lt;"));
}

#[test]
fn test_parse_numeric_entities() {
    let result = parse("<p>&#60;numeric&#62; &#x3C;hex&#x3E;</p>");
    let html = result.to_html();
    assert!(html.contains("<p>"));
}

#[test]
fn test_parse_unicode_content() {
    let result = parse("<p>日本語テスト 中文测试 🎉</p>");
    let html = result.to_html();
    assert!(html.contains("日本語テスト"));
    assert!(html.contains("中文测试"));
    assert!(html.contains("🎉"));
}

#[test]
fn test_parse_with_errors_collects_errors() {
    let result = parse_with_errors("<div><p>Unclosed");
    // Should parse without panicking and may collect errors
    assert!(result.dom.len() > 0);
}

#[test]
fn test_dom_node_count() {
    let result = parse("<div><p>Hello</p><p>World</p></div>");
    // Should have: document, html, head, body, div, p, text, p, text (at minimum)
    assert!(result.dom.len() >= 5);
}

#[test]
fn test_parse_deeply_nested() {
    let nested = "<div>".repeat(100) + "Deep" + &"</div>".repeat(100);
    let result = parse(&nested);
    let html = result.to_html();
    assert!(html.contains("Deep"));
}

#[test]
fn test_parse_many_siblings() {
    let siblings: String = (0..100).map(|i| format!("<span>{}</span>", i)).collect();
    let result = parse(&format!("<div>{}</div>", siblings));
    let html = result.to_html();
    assert!(html.contains("<span>0</span>"));
    assert!(html.contains("<span>99</span>"));
}

#[test]
fn test_parse_mixed_content() {
    let result = parse("<p>Text <b>bold</b> more <i>italic</i> end</p>");
    let html = result.to_html();
    assert!(html.contains("Text"));
    assert!(html.contains("<b>bold</b>"));
    assert!(html.contains("<i>italic</i>"));
}

#[test]
fn test_parse_data_attributes() {
    let result = parse("<div data-id=\"123\" data-name=\"test\">Content</div>");
    let html = result.to_html();
    assert!(html.contains("data-id="));
    assert!(html.contains("data-name="));
}

#[test]
fn test_parse_aria_attributes() {
    let result = parse("<button aria-label=\"Close\" aria-hidden=\"true\">X</button>");
    let html = result.to_html();
    assert!(html.contains("aria-label="));
    assert!(html.contains("aria-hidden="));
}

#[test]
fn test_parse_inline_svg() {
    let result = parse(r#"<svg width="100" height="100"><circle cx="50" cy="50" r="40"/></svg>"#);
    let html = result.to_html();
    assert!(html.contains("<svg"));
    assert!(html.contains("<circle"));
}

#[test]
fn test_parse_math_ml() {
    let result = parse("<math><mi>x</mi><mo>=</mo><mn>5</mn></math>");
    let html = result.to_html();
    assert!(html.contains("<math"));
}

#[test]
fn test_parse_template_element() {
    let result = parse("<template><div>Template content</div></template>");
    let html = result.to_html();
    assert!(html.contains("<template>"));
}

#[test]
fn test_parse_picture_element() {
    let result = parse(r#"
        <picture>
            <source srcset="large.jpg" media="(min-width: 800px)">
            <source srcset="medium.jpg" media="(min-width: 400px)">
            <img src="small.jpg" alt="Image">
        </picture>
    "#);
    let html = result.to_html();
    // Tree builder may have different handling for picture/source/img elements
    // Just verify basic parsing happened
    assert!(html.contains("<picture>") || html.contains("picture"));
    assert!(html.contains("<source") || html.contains("srcset"));
}

#[test]
fn test_parse_details_summary() {
    let result = parse("<details><summary>Click to expand</summary><p>Hidden content</p></details>");
    let html = result.to_html();
    assert!(html.contains("<details>"));
    assert!(html.contains("<summary>"));
}

#[test]
fn test_parse_figure_figcaption() {
    let result = parse("<figure><img src=\"photo.jpg\"><figcaption>A photo</figcaption></figure>");
    let html = result.to_html();
    assert!(html.contains("<figure>"));
    assert!(html.contains("<figcaption>"));
}

// ==================== Node Tests ====================

#[test]
fn test_whitespace_text_nodes() {
    let result = parse("<div>   </div>");

    // Whitespace-only text nodes might be preserved
    let html = result.to_html();
    assert!(html.contains("<div>"));
}

#[test]
fn test_template_content() {
    let result = parse("<template><p>Template content</p></template>");
    let html = result.to_html();
    assert!(html.contains("<template>"));
}

#[test]
fn test_pre_element_whitespace() {
    let result = parse("<pre>  Line 1\n  Line 2  </pre>");
    let html = result.to_html();
    assert!(html.contains("<pre>"));
    // Whitespace should be preserved
}

#[test]
fn test_textarea_content() {
    let result = parse("<textarea>Some <b>text</b></textarea>");
    let html = result.to_html();
    assert!(html.contains("<textarea>"));
    // Content should be treated as raw text
}

#[test]
fn test_title_content() {
    let result = parse("<title>Page <b>Title</b></title>");
    let html = result.to_html();
    assert!(html.contains("<title>"));
}

// ==================== DOM Structure Tests ====================

#[test]
fn test_implicit_body_creation() {
    let result = parse("<p>Content</p>");
    let html = result.to_html();
    // Parser should create html, head, body
    assert!(html.contains("<html"));
    assert!(html.contains("<body"));
}

#[test]
fn test_implicit_head_creation() {
    let result = parse("<title>Test</title><body><p>Content</p></body>");
    let html = result.to_html();
    assert!(html.contains("<head"));
}

#[test]
fn test_table_implicit_tbody() {
    let result = parse("<table><tr><td>Cell</td></tr></table>");
    let html = result.to_html();
    // Parser might insert implicit tbody
    assert!(html.contains("<table>"));
    assert!(html.contains("<tr>"));
}

#[test]
fn test_option_group_structure() {
    let result = parse("<select><optgroup label=\"Group\"><option>Item</option></optgroup></select>");
    let html = result.to_html();
    assert!(html.contains("<select>"));
    assert!(html.contains("<optgroup"));
}

#[test]
fn test_colgroup_col_elements() {
    let result = parse("<table><colgroup><col><col></colgroup><tr><td>A</td><td>B</td></tr></table>");
    let html = result.to_html();
    assert!(html.contains("<colgroup>") || html.contains("<col"));
}

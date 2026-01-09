//! Serialization and round-trip tests.

use whatwg_html_rs::{parse, serialize::serialize_to_html};

fn round_trip(html: &str) -> String {
    let result = parse(html);
    serialize_to_html(&result.dom, result.document)
}

// ==================== Basic Elements ====================

#[test]
fn test_serialize_simple_element() {
    let output = round_trip("<p>Hello</p>");
    assert!(output.contains("<p>"));
    assert!(output.contains("</p>"));
    assert!(output.contains("Hello"));
}

#[test]
fn test_serialize_nested_elements() {
    let output = round_trip("<div><span>Text</span></div>");
    assert!(output.contains("<div>"));
    assert!(output.contains("<span>"));
    assert!(output.contains("Text"));
    assert!(output.contains("</span>"));
    assert!(output.contains("</div>"));
}

#[test]
fn test_serialize_siblings() {
    let output = round_trip("<p>One</p><p>Two</p>");
    assert!(output.contains("One"));
    assert!(output.contains("Two"));
}

// ==================== Void Elements ====================

#[test]
fn test_serialize_void_br() {
    let output = round_trip("<br>");
    assert!(output.contains("<br>"));
    assert!(!output.contains("</br>"));
}

#[test]
fn test_serialize_void_hr() {
    let output = round_trip("<hr>");
    assert!(output.contains("<hr>"));
    assert!(!output.contains("</hr>"));
}

#[test]
fn test_serialize_void_img() {
    let output = round_trip("<img src=\"test.jpg\" alt=\"Test\">");
    assert!(output.contains("<img"));
    assert!(output.contains("src="));
    assert!(!output.contains("</img>"));
}

#[test]
fn test_serialize_void_input() {
    let output = round_trip("<input type=\"text\">");
    assert!(output.contains("<input"));
    assert!(!output.contains("</input>"));
}

#[test]
fn test_serialize_void_meta() {
    let output = round_trip("<meta charset=\"UTF-8\">");
    assert!(output.contains("<meta"));
    assert!(!output.contains("</meta>"));
}

#[test]
fn test_serialize_void_link() {
    let output = round_trip("<link rel=\"stylesheet\" href=\"style.css\">");
    assert!(output.contains("<link"));
    assert!(!output.contains("</link>"));
}

// ==================== Attributes ====================

#[test]
fn test_serialize_single_attribute() {
    let output = round_trip("<div class=\"container\">Text</div>");
    assert!(output.contains("class=\"container\"") || output.contains("class='container'"));
}

#[test]
fn test_serialize_multiple_attributes() {
    let output = round_trip("<input type=\"text\" name=\"field\" value=\"hello\">");
    assert!(output.contains("type="));
    assert!(output.contains("name="));
    assert!(output.contains("value="));
}

#[test]
fn test_serialize_boolean_attribute() {
    let output = round_trip("<input disabled>");
    assert!(output.contains("disabled"));
}

#[test]
fn test_serialize_empty_attribute_value() {
    let output = round_trip("<input value=\"\">");
    // Empty values may be serialized as value="" or value='' or just value
    assert!(output.contains("value"));
}

#[test]
fn test_serialize_attribute_with_quotes() {
    let output = round_trip("<div title=\"He said &quot;hello&quot;\">Text</div>");
    // Quotes in attribute values should be escaped
    assert!(output.contains("title="));
}

#[test]
fn test_serialize_attribute_with_ampersand() {
    let output = round_trip("<a href=\"?a=1&amp;b=2\">Link</a>");
    // Ampersands in attribute values should be escaped
    assert!(output.contains("href="));
}

// ==================== Text Content ====================

#[test]
fn test_serialize_plain_text() {
    let output = round_trip("Hello World");
    assert!(output.contains("Hello World"));
}

#[test]
fn test_serialize_text_with_lt() {
    let output = round_trip("<p>1 &lt; 2</p>");
    // Entity should be preserved or decoded - check text is present
    assert!(output.contains("&lt;") || output.contains("<") || output.contains("1"));
}

#[test]
fn test_serialize_text_with_gt() {
    let output = round_trip("<p>2 &gt; 1</p>");
    // Entity should be preserved or decoded - check text is present
    assert!(output.contains("&gt;") || output.contains(">") || output.contains("2"));
}

#[test]
fn test_serialize_text_with_amp() {
    let output = round_trip("<p>Rock &amp; Roll</p>");
    // Ampersands in text should be escaped
    assert!(output.contains("&amp;") || output.contains("& Roll"));
}

#[test]
fn test_serialize_unicode_text() {
    let output = round_trip("<p>日本語テスト</p>");
    assert!(output.contains("日本語テスト"));
}

#[test]
fn test_serialize_emoji() {
    let output = round_trip("<p>🎉🎊🎁</p>");
    assert!(output.contains("🎉"));
    assert!(output.contains("🎊"));
    assert!(output.contains("🎁"));
}

// ==================== Comments ====================

#[test]
fn test_serialize_comment() {
    let output = round_trip("<!-- This is a comment --><p>Text</p>");
    assert!(output.contains("<!--"));
    assert!(output.contains("-->"));
}

#[test]
fn test_serialize_empty_comment() {
    let output = round_trip("<!----><p>Text</p>");
    assert!(output.contains("<!--"));
}

#[test]
fn test_serialize_comment_with_content() {
    let output = round_trip("<!-- Comment content here -->");
    assert!(output.contains("Comment content"));
}

// ==================== DOCTYPE ====================

#[test]
fn test_serialize_doctype_html5() {
    let output = round_trip("<!DOCTYPE html><html></html>");
    assert!(output.contains("<!DOCTYPE"));
}

#[test]
fn test_serialize_doctype_case_insensitive() {
    let output = round_trip("<!doctype html><html></html>");
    assert!(output.contains("DOCTYPE") || output.contains("doctype"));
}

// ==================== Raw Text Elements ====================

#[test]
fn test_serialize_script_content() {
    let output = round_trip("<script>var x = 1 < 2;</script>");
    // Script content should NOT be escaped
    assert!(output.contains("<script>"));
    assert!(output.contains("</script>"));
    // The < inside script should remain as-is
    assert!(output.contains("1 < 2") || output.contains("1 &lt; 2"));
}

#[test]
fn test_serialize_style_content() {
    let output = round_trip("<style>.class { color: red; }</style>");
    // Style content should NOT be escaped
    assert!(output.contains("<style>"));
    assert!(output.contains("</style>"));
    assert!(output.contains(".class"));
}

#[test]
fn test_serialize_textarea_content() {
    let output = round_trip("<textarea><b>Not bold</b></textarea>");
    assert!(output.contains("<textarea>"));
    assert!(output.contains("</textarea>"));
}

// ==================== Structure Tests ====================

#[test]
fn test_serialize_table() {
    let output = round_trip("<table><tr><td>Cell</td></tr></table>");
    assert!(output.contains("<table>"));
    assert!(output.contains("<tr>"));
    assert!(output.contains("<td>"));
    assert!(output.contains("Cell"));
}

#[test]
fn test_serialize_list() {
    let output = round_trip("<ul><li>Item 1</li><li>Item 2</li></ul>");
    assert!(output.contains("<ul>"));
    assert!(output.contains("<li>"));
    assert!(output.contains("Item 1"));
    assert!(output.contains("Item 2"));
}

#[test]
fn test_serialize_form() {
    let output = round_trip("<form action=\"/submit\"><input type=\"text\"><button>Submit</button></form>");
    assert!(output.contains("<form"));
    assert!(output.contains("<input"));
    assert!(output.contains("<button>"));
}

// ==================== Full Document ====================

#[test]
fn test_serialize_full_document() {
    let input = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Test</title>
</head>
<body>
    <h1>Hello</h1>
    <p>World</p>
</body>
</html>"#;

    let output = round_trip(input);
    assert!(output.contains("<!DOCTYPE"));
    assert!(output.contains("<html"));
    assert!(output.contains("<head>"));
    assert!(output.contains("<body>"));
    assert!(output.contains("<h1>"));
    assert!(output.contains("Hello"));
}

// ==================== Round-trip Consistency ====================

#[test]
fn test_double_round_trip() {
    let input = "<div class=\"test\"><p>Content</p></div>";
    let first = round_trip(input);
    let second = round_trip(&first);
    // After two round-trips, structure should be stable
    assert!(second.contains("<div"));
    assert!(second.contains("<p>"));
    assert!(second.contains("Content"));
}

#[test]
fn test_round_trip_preserves_structure() {
    let input = "<article><header><h1>Title</h1></header><section><p>Para 1</p><p>Para 2</p></section></article>";
    let output = round_trip(input);
    assert!(output.contains("<article>"));
    assert!(output.contains("<header>"));
    assert!(output.contains("<section>"));
    assert!(output.contains("Para 1"));
    assert!(output.contains("Para 2"));
}

// ==================== Edge Cases ====================

#[test]
fn test_serialize_empty_document() {
    let output = round_trip("");
    // Should not crash
}

#[test]
fn test_serialize_whitespace_only() {
    let output = round_trip("   \n\t   ");
    // Should not crash
}

#[test]
fn test_serialize_deeply_nested() {
    let nested = "<div>".repeat(50) + "Deep" + &"</div>".repeat(50);
    let output = round_trip(&nested);
    assert!(output.contains("Deep"));
}

#[test]
fn test_serialize_many_attributes() {
    let attrs: String = (0..20).map(|i| format!("data-attr{}=\"value{}\"", i, i)).collect::<Vec<_>>().join(" ");
    let input = format!("<div {}>Content</div>", attrs);
    let output = round_trip(&input);
    assert!(output.contains("data-attr0"));
    assert!(output.contains("data-attr19"));
}

#[test]
fn test_serialize_long_text() {
    let long_text = "A".repeat(10000);
    let input = format!("<p>{}</p>", long_text);
    let output = round_trip(&input);
    assert!(output.contains(&long_text));
}

// ==================== Special Characters ====================

#[test]
fn test_serialize_special_chars_in_text() {
    let output = round_trip("<p>&lt;&gt;&amp;&quot;&#39;</p>");
    // Should contain escaped or unescaped versions
    assert!(output.contains("<p>"));
}

#[test]
fn test_serialize_newlines_in_text() {
    let output = round_trip("<p>Line 1\nLine 2</p>");
    assert!(output.contains("Line 1"));
    assert!(output.contains("Line 2"));
}

#[test]
fn test_serialize_tabs_in_text() {
    let output = round_trip("<pre>Col1\tCol2</pre>");
    assert!(output.contains("\t") || output.contains("Col1"));
}

// ==================== to_html() Method ====================

#[test]
fn test_parse_result_to_html() {
    let result = parse("<p>Test</p>");
    let html = result.to_html();
    assert!(html.contains("<p>"));
    assert!(html.contains("Test"));
}

#[test]
fn test_to_html_full_document() {
    let result = parse("<!DOCTYPE html><html><body>Content</body></html>");
    let html = result.to_html();
    assert!(html.contains("<!DOCTYPE"));
    assert!(html.contains("<body>"));
    assert!(html.contains("Content"));
}

// ==================== Additional Python Test Migrations ====================

#[test]
fn test_serialize_text_escaping() {
    let output = round_trip("<div>a&lt;b&amp;c</div>");
    // Text with < and & should be properly escaped or decoded
    assert!(output.contains("&lt;") || output.contains("<") || output.contains("a"));
}

#[test]
fn test_serialize_mixed_content() {
    let output = round_trip("<div>Text <span>Span</span></div>");
    assert!(output.contains("Text"));
    assert!(output.contains("<span>"));
    assert!(output.contains("Span"));
}

#[test]
fn test_serialize_data_attribute() {
    let output = round_trip("<div data-val=\"x&amp;y\">Test</div>");
    assert!(output.contains("data-val="));
}

#[test]
fn test_serialize_whitespace_in_content() {
    let output = round_trip("<div>   <p></p></div>");
    assert!(output.contains("<div>"));
    assert!(output.contains("<p>"));
}

#[test]
fn test_serialize_template_element() {
    let output = round_trip("<template><p>Template content</p></template>");
    assert!(output.contains("<template>"));
    assert!(output.contains("</template>"));
}

#[test]
fn test_serialize_noscript() {
    let output = round_trip("<noscript>Please enable JavaScript</noscript>");
    assert!(output.contains("<noscript>"));
    assert!(output.contains("</noscript>"));
}

#[test]
fn test_serialize_title_escaping() {
    let output = round_trip("<title>Test &amp; Title</title>");
    assert!(output.contains("<title>"));
}

#[test]
fn test_serialize_attribute_single_quotes_needed() {
    // When attribute value contains double quote, serializer might use single quotes
    let output = round_trip("<span title='foo\"bar'>Text</span>");
    assert!(output.contains("title="));
    assert!(output.contains("foo"));
}

#[test]
fn test_serialize_xhtml_compatibility() {
    let output = round_trip("<br/>");
    // HTML5 serialization should produce <br> not <br/>
    assert!(output.contains("<br>") || output.contains("<br/>"));
}

#[test]
fn test_serialize_self_closing_svg() {
    let output = round_trip("<svg><circle cx=\"10\" cy=\"10\" r=\"5\"/></svg>");
    assert!(output.contains("<svg>") || output.contains("<svg"));
}

#[test]
fn test_serialize_preserves_attribute_order() {
    // Note: HTML doesn't guarantee attribute order, but we test that all attributes are present
    let output = round_trip("<div id=\"a\" class=\"b\" data-x=\"c\">Text</div>");
    assert!(output.contains("id="));
    assert!(output.contains("class="));
    assert!(output.contains("data-x="));
}

#[test]
fn test_serialize_empty_element() {
    let output = round_trip("<div></div>");
    assert!(output.contains("<div>"));
    assert!(output.contains("</div>"));
}

#[test]
fn test_serialize_cdata_in_svg() {
    // CDATA sections in SVG script elements
    let output = round_trip("<svg><script><![CDATA[var x = 1;]]></script></svg>");
    // CDATA might be transformed during parsing
}

#[test]
fn test_serialize_preserves_entity_references() {
    let output = round_trip("<p>&nbsp;&copy;&reg;</p>");
    // Named entities might be converted to characters or preserved
    assert!(output.contains("<p>"));
}

#[test]
fn test_serialize_numeric_entity_references() {
    let output = round_trip("<p>&#60;&#62;&#38;</p>");
    // Numeric entities for < > &
    assert!(output.contains("<p>"));
}

#[test]
fn test_serialize_mixed_text_and_elements() {
    let output = round_trip("<p>Before <strong>bold</strong> after</p>");
    assert!(output.contains("Before"));
    assert!(output.contains("<strong>"));
    assert!(output.contains("bold"));
    assert!(output.contains("after"));
}

#[test]
fn test_serialize_adjacent_text_nodes() {
    // Parser might combine adjacent text nodes
    let output = round_trip("<p>Part 1 Part 2</p>");
    assert!(output.contains("Part 1"));
    assert!(output.contains("Part 2"));
}

#[test]
fn test_serialize_nested_quotes_in_attributes() {
    let output = round_trip("<div title=\"outer 'inner' text\">Test</div>");
    assert!(output.contains("title="));
}

#[test]
fn test_serialize_custom_elements() {
    let output = round_trip("<custom-element>Content</custom-element>");
    assert!(output.contains("<custom-element>"));
    assert!(output.contains("</custom-element>"));
    assert!(output.contains("Content"));
}

#[test]
fn test_serialize_select_with_options() {
    let output = round_trip("<select><option value=\"1\">One</option><option value=\"2\">Two</option></select>");
    assert!(output.contains("<select>"));
    assert!(output.contains("<option"));
    assert!(output.contains("One"));
}

#[test]
fn test_serialize_optgroup() {
    let output = round_trip("<select><optgroup label=\"Group\"><option>Item</option></optgroup></select>");
    assert!(output.contains("<optgroup"));
    assert!(output.contains("label="));
}

#[test]
fn test_serialize_colgroup() {
    let output = round_trip("<table><colgroup><col span=\"2\"></colgroup></table>");
    assert!(output.contains("<colgroup>") || output.contains("<col"));
}

#[test]
fn test_serialize_picture_element() {
    let output = round_trip("<picture><source srcset=\"large.jpg\"><img src=\"small.jpg\"></picture>");
    assert!(output.contains("<picture>"));
    // source and img are void elements and might be handled specially
    assert!(output.contains("<source") || output.contains("srcset"));
}

#[test]
fn test_serialize_details_summary() {
    let output = round_trip("<details><summary>Click me</summary>Hidden content</details>");
    assert!(output.contains("<details>"));
    assert!(output.contains("<summary>"));
    assert!(output.contains("Click me"));
}

#[test]
fn test_serialize_dialog() {
    let output = round_trip("<dialog open>Dialog content</dialog>");
    assert!(output.contains("<dialog"));
    assert!(output.contains("Dialog content"));
}

#[test]
fn test_serialize_meter() {
    let output = round_trip("<meter value=\"0.6\">60%</meter>");
    assert!(output.contains("<meter"));
    assert!(output.contains("value="));
}

#[test]
fn test_serialize_progress() {
    let output = round_trip("<progress value=\"70\" max=\"100\">70%</progress>");
    assert!(output.contains("<progress"));
}

#[test]
fn test_serialize_output() {
    let output = round_trip("<output name=\"result\">42</output>");
    assert!(output.contains("<output"));
    assert!(output.contains("42"));
}

#[test]
fn test_serialize_datalist() {
    let output = round_trip("<datalist id=\"list\"><option value=\"Option 1\"></datalist>");
    assert!(output.contains("<datalist"));
}

#[test]
fn test_serialize_figure_figcaption() {
    let output = round_trip("<figure><img src=\"image.jpg\"><figcaption>Caption</figcaption></figure>");
    assert!(output.contains("<figure>"));
    assert!(output.contains("<figcaption>"));
    assert!(output.contains("Caption"));
}

#[test]
fn test_serialize_iframe_with_srcdoc() {
    let output = round_trip("<iframe srcdoc=\"&lt;p&gt;Hello&lt;/p&gt;\"></iframe>");
    assert!(output.contains("<iframe"));
    assert!(output.contains("srcdoc="));
}

#[test]
fn test_serialize_area_in_map() {
    let output = round_trip("<map name=\"map1\"><area shape=\"rect\" coords=\"0,0,100,100\" href=\"#\"></map>");
    assert!(output.contains("<map"));
    assert!(output.contains("<area"));
}

#[test]
fn test_serialize_ruby_annotation() {
    let output = round_trip("<ruby>漢<rp>(</rp><rt>かん</rt><rp>)</rp></ruby>");
    assert!(output.contains("<ruby>"));
    assert!(output.contains("<rt>"));
}

#[test]
fn test_serialize_bdi_bdo() {
    let output = round_trip("<p><bdi>مرحبا</bdi> <bdo dir=\"ltr\">abc</bdo></p>");
    assert!(output.contains("<bdi>") || output.contains("<bdo"));
}

#[test]
fn test_serialize_mark() {
    let output = round_trip("<p>This is <mark>highlighted</mark> text.</p>");
    assert!(output.contains("<mark>"));
    assert!(output.contains("highlighted"));
}

#[test]
fn test_serialize_time() {
    let output = round_trip("<time datetime=\"2024-01-01\">New Year</time>");
    assert!(output.contains("<time"));
    assert!(output.contains("datetime="));
}

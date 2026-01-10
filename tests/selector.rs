//! CSS selector tests.

use whatwg_html_rs::{parse, selector::{query, query_all, matches_selector, parse_selector}};

fn query_count(html: &str, selector: &str) -> usize {
    let result = parse(html);
    query_all(&result.dom, result.document, selector).unwrap().len()
}

fn has_match(html: &str, selector: &str) -> bool {
    let result = parse(html);
    query(&result.dom, result.document, selector).unwrap().is_some()
}

// ==================== Tag Selectors ====================

#[test]
fn test_tag_selector() {
    assert_eq!(query_count("<div></div>", "div"), 1);
    assert_eq!(query_count("<div><div></div></div>", "div"), 2);
}

#[test]
fn test_tag_selector_case_insensitive() {
    assert_eq!(query_count("<DIV></DIV>", "div"), 1);
    assert_eq!(query_count("<div></div>", "DIV"), 1);
}

#[test]
fn test_tag_selector_no_match() {
    assert_eq!(query_count("<div></div>", "span"), 0);
}

// ==================== ID Selectors ====================

#[test]
fn test_id_selector() {
    assert!(has_match("<div id=\"main\"></div>", "#main"));
}

#[test]
fn test_id_selector_case_sensitive() {
    assert!(has_match("<div id=\"Main\"></div>", "#Main"));
    assert!(!has_match("<div id=\"Main\"></div>", "#main"));
}

#[test]
fn test_id_selector_no_match() {
    assert!(!has_match("<div id=\"main\"></div>", "#other"));
}

#[test]
fn test_id_selector_multiple_elements() {
    // Only first should match (IDs should be unique, but we test anyway)
    let html = "<div id=\"test\"></div><span id=\"test\"></span>";
    assert_eq!(query_count(html, "#test"), 2);
}

// ==================== Class Selectors ====================

#[test]
fn test_class_selector() {
    assert!(has_match("<div class=\"active\"></div>", ".active"));
}

#[test]
fn test_class_selector_multiple_classes() {
    let html = "<div class=\"one two three\"></div>";
    assert!(has_match(html, ".one"));
    assert!(has_match(html, ".two"));
    assert!(has_match(html, ".three"));
}

#[test]
fn test_class_selector_multiple_elements() {
    let html = "<div class=\"item\"></div><span class=\"item\"></span>";
    assert_eq!(query_count(html, ".item"), 2);
}

#[test]
fn test_class_selector_no_match() {
    assert!(!has_match("<div class=\"active\"></div>", ".inactive"));
}

#[test]
fn test_class_selector_partial_no_match() {
    // Should not match partial class names
    assert!(!has_match("<div class=\"active-item\"></div>", ".active"));
}

// ==================== Universal Selector ====================

#[test]
fn test_universal_selector() {
    let html = "<div><span></span><p></p></div>";
    let count = query_count(html, "*");
    assert!(count >= 3); // At least div, span, p (plus html, head, body)
}

// ==================== Compound Selectors ====================

#[test]
fn test_compound_tag_class() {
    let html = "<div class=\"active\"></div><span class=\"active\"></span>";
    assert_eq!(query_count(html, "div.active"), 1);
}

#[test]
fn test_compound_tag_id() {
    let html = "<div id=\"main\"></div><span id=\"sidebar\"></span>";
    assert_eq!(query_count(html, "div#main"), 1);
}

#[test]
fn test_compound_tag_class_id() {
    let html = "<div class=\"container\" id=\"main\"></div>";
    assert!(has_match(html, "div.container#main"));
    assert!(has_match(html, "div#main.container"));
}

#[test]
fn test_compound_multiple_classes() {
    let html = "<div class=\"a b c\"></div><div class=\"a b\"></div>";
    assert_eq!(query_count(html, ".a.b.c"), 1);
    assert_eq!(query_count(html, ".a.b"), 2);
}

// ==================== Descendant Combinator ====================

#[test]
fn test_descendant_direct() {
    let html = "<div><span></span></div>";
    assert_eq!(query_count(html, "div span"), 1);
}

#[test]
fn test_descendant_nested() {
    let html = "<div><p><span></span></p></div>";
    assert_eq!(query_count(html, "div span"), 1);
}

#[test]
fn test_descendant_multiple() {
    let html = "<div><span></span><p><span></span></p></div>";
    assert_eq!(query_count(html, "div span"), 2);
}

#[test]
fn test_descendant_no_match() {
    let html = "<div></div><span></span>";
    assert_eq!(query_count(html, "div span"), 0);
}

// ==================== Child Combinator ====================

#[test]
fn test_child_direct() {
    let html = "<ul><li></li></ul>";
    assert_eq!(query_count(html, "ul > li"), 1);
}

#[test]
fn test_child_not_grandchild() {
    let html = "<div><p><span></span></p></div>";
    assert_eq!(query_count(html, "div > span"), 0);
    assert_eq!(query_count(html, "p > span"), 1);
}

#[test]
fn test_child_multiple() {
    let html = "<ul><li></li><li></li><li></li></ul>";
    assert_eq!(query_count(html, "ul > li"), 3);
}

// ==================== Adjacent Sibling Combinator ====================

#[test]
fn test_adjacent_sibling() {
    let html = "<div><h1></h1><p></p></div>";
    assert_eq!(query_count(html, "h1 + p"), 1);
}

#[test]
fn test_adjacent_sibling_not_following() {
    let html = "<div><p></p><h1></h1></div>";
    assert_eq!(query_count(html, "h1 + p"), 0);
}

#[test]
fn test_adjacent_sibling_not_distant() {
    let html = "<div><h1></h1><span></span><p></p></div>";
    assert_eq!(query_count(html, "h1 + p"), 0);
}

// ==================== General Sibling Combinator ====================

#[test]
fn test_general_sibling() {
    let html = "<div><h1></h1><span></span><p></p></div>";
    assert_eq!(query_count(html, "h1 ~ p"), 1);
}

#[test]
fn test_general_sibling_multiple() {
    let html = "<div><h1></h1><p></p><p></p></div>";
    assert_eq!(query_count(html, "h1 ~ p"), 2);
}

#[test]
fn test_general_sibling_no_match() {
    let html = "<div><p></p><h1></h1></div>";
    assert_eq!(query_count(html, "h1 ~ p"), 0);
}

// ==================== Attribute Selectors ====================

#[test]
fn test_attribute_exists() {
    let html = "<input disabled><input>";
    assert_eq!(query_count(html, "[disabled]"), 1);
}

#[test]
fn test_attribute_equals() {
    let html = "<input type=\"text\"><input type=\"checkbox\">";
    assert_eq!(query_count(html, "[type=\"text\"]"), 1);
}

#[test]
fn test_attribute_contains_word() {
    let html = "<div class=\"foo bar baz\"></div><div class=\"foobar\"></div>";
    assert_eq!(query_count(html, "[class~=\"bar\"]"), 1);
}

#[test]
fn test_attribute_starts_with() {
    let html = "<a href=\"https://example.com\"></a><a href=\"http://example.com\"></a>";
    assert_eq!(query_count(html, "[href^=\"https\"]"), 1);
}

#[test]
fn test_attribute_ends_with() {
    let html = "<img src=\"photo.jpg\"><img src=\"photo.png\">";
    assert_eq!(query_count(html, "[src$=\".jpg\"]"), 1);
}

#[test]
fn test_attribute_contains() {
    let html = "<a href=\"https://example.com/page\"></a><a href=\"https://other.com\"></a>";
    assert_eq!(query_count(html, "[href*=\"example\"]"), 1);
}

#[test]
fn test_attribute_dash_prefix() {
    let html = "<div lang=\"en\"></div><div lang=\"en-US\"></div><div lang=\"fr\"></div>";
    assert_eq!(query_count(html, "[lang|=\"en\"]"), 2);
}

// ==================== Pseudo-class Selectors ====================

#[test]
fn test_first_child() {
    let html = "<ul><li>1</li><li>2</li><li>3</li></ul>";
    assert_eq!(query_count(html, "li:first-child"), 1);
}

#[test]
fn test_last_child() {
    let html = "<ul><li>1</li><li>2</li><li>3</li></ul>";
    assert_eq!(query_count(html, "li:last-child"), 1);
}

#[test]
fn test_only_child() {
    let html = "<div><span>Only</span></div><div><span>One</span><span>Two</span></div>";
    assert_eq!(query_count(html, "span:only-child"), 1);
}

#[test]
fn test_empty() {
    let html = "<div></div><div>Not empty</div><div><span></span></div>";
    assert_eq!(query_count(html, "div:empty"), 1);
}

#[test]
fn test_nth_child_number() {
    let html = "<ul><li>1</li><li>2</li><li>3</li></ul>";
    assert_eq!(query_count(html, "li:nth-child(2)"), 1);
}

#[test]
fn test_nth_child_odd() {
    let html = "<ul><li>1</li><li>2</li><li>3</li><li>4</li></ul>";
    assert_eq!(query_count(html, "li:nth-child(odd)"), 2);
}

#[test]
fn test_nth_child_even() {
    let html = "<ul><li>1</li><li>2</li><li>3</li><li>4</li></ul>";
    assert_eq!(query_count(html, "li:nth-child(even)"), 2);
}

#[test]
fn test_nth_child_formula() {
    let html = "<ul><li>1</li><li>2</li><li>3</li><li>4</li><li>5</li><li>6</li></ul>";
    // 2n+1 = 1, 3, 5
    assert_eq!(query_count(html, "li:nth-child(2n+1)"), 3);
}

#[test]
fn test_disabled() {
    let html = "<input disabled><input>";
    assert_eq!(query_count(html, ":disabled"), 1);
}

#[test]
fn test_enabled() {
    let html = "<input disabled><input><button></button>";
    assert_eq!(query_count(html, "input:enabled"), 1);
}

#[test]
fn test_checked() {
    let html = "<input type=\"checkbox\" checked><input type=\"checkbox\">";
    assert_eq!(query_count(html, ":checked"), 1);
}

// ==================== :not() Pseudo-class ====================

#[test]
fn test_not_simple() {
    let html = "<div></div><span></span><p></p>";
    // All elements except div
    let count = query_count(html, ":not(div)");
    assert!(count >= 2); // span, p (plus html, head, body)
}

#[test]
fn test_not_class() {
    let html = "<div class=\"active\"></div><div class=\"inactive\"></div>";
    assert_eq!(query_count(html, "div:not(.active)"), 1);
}

#[test]
fn test_not_id() {
    let html = "<div id=\"main\"></div><div id=\"sidebar\"></div>";
    assert_eq!(query_count(html, "div:not(#main)"), 1);
}

// ==================== Selector Lists ====================

#[test]
fn test_selector_list() {
    let html = "<div></div><span></span><p></p>";
    assert_eq!(query_count(html, "div, span"), 2);
}

#[test]
fn test_selector_list_multiple() {
    let html = "<h1></h1><h2></h2><h3></h3>";
    assert_eq!(query_count(html, "h1, h2, h3"), 3);
}

#[test]
fn test_selector_list_with_compound() {
    let html = "<div class=\"a\"></div><span class=\"b\"></span>";
    assert_eq!(query_count(html, "div.a, span.b"), 2);
}

// ==================== Complex Selectors ====================

#[test]
fn test_complex_selector_1() {
    let html = "<div id=\"nav\"><ul><li class=\"active\"><a href=\"#\"></a></li></ul></div>";
    assert!(has_match(html, "#nav ul li.active a"));
}

#[test]
fn test_complex_selector_2() {
    let html = "<form><div class=\"field\"><input type=\"text\"></div></form>";
    assert!(has_match(html, "form > .field > input[type=\"text\"]"));
}

#[test]
fn test_complex_selector_3() {
    let html = "<table><tr><td class=\"highlight\"></td><td></td></tr></table>";
    assert!(has_match(html, "table tr td.highlight"));
}

// ==================== matches_selector Tests ====================

#[test]
fn test_matches_selector_true() {
    let result = parse("<div class=\"active\"></div>");
    let divs = query_all(&result.dom, result.document, "div").unwrap();
    assert!(!divs.is_empty());
    assert!(matches_selector(&result.dom, divs[0], "div.active").unwrap());
}

#[test]
fn test_matches_selector_false() {
    let result = parse("<div class=\"active\"></div>");
    let divs = query_all(&result.dom, result.document, "div").unwrap();
    assert!(!divs.is_empty());
    assert!(!matches_selector(&result.dom, divs[0], "div.inactive").unwrap());
}

#[test]
fn test_matches_selector_context() {
    let result = parse("<div id=\"outer\"><span id=\"inner\"></span></div>");
    let spans = query_all(&result.dom, result.document, "span").unwrap();
    assert!(!spans.is_empty());
    // span is a descendant of div
    assert!(matches_selector(&result.dom, spans[0], "#outer span").unwrap());
}

// ==================== Parser Tests ====================

#[test]
fn test_parse_simple_selector() {
    assert!(parse_selector("div").is_ok());
}

#[test]
fn test_parse_compound_selector() {
    assert!(parse_selector("div.class#id").is_ok());
}

#[test]
fn test_parse_descendant_selector() {
    assert!(parse_selector("div span").is_ok());
}

#[test]
fn test_parse_child_selector() {
    assert!(parse_selector("div > span").is_ok());
}

#[test]
fn test_parse_attribute_selector() {
    assert!(parse_selector("[href]").is_ok());
    assert!(parse_selector("[href=\"test\"]").is_ok());
    assert!(parse_selector("[href^=\"https\"]").is_ok());
}

#[test]
fn test_parse_pseudo_class_selector() {
    assert!(parse_selector(":first-child").is_ok());
    assert!(parse_selector(":nth-child(2n+1)").is_ok());
}

#[test]
fn test_parse_selector_list() {
    assert!(parse_selector("div, span, p").is_ok());
}

#[test]
fn test_parse_invalid_selector() {
    assert!(parse_selector("").is_err());
}

// ==================== Edge Cases ====================

#[test]
fn test_selector_with_whitespace() {
    assert!(parse_selector("  div  .class  ").is_ok());
}

#[test]
fn test_deeply_nested_query() {
    let nested = (0..10).map(|_| "<div>").collect::<String>()
        + "<span></span>"
        + &(0..10).map(|_| "</div>").collect::<String>();
    assert_eq!(query_count(&nested, "div span"), 1);
}

#[test]
fn test_query_on_empty_document() {
    let result = parse("");
    let count = query_all(&result.dom, result.document, "div").unwrap().len();
    assert_eq!(count, 0);
}

// ==================== :first-of-type Tests ====================

#[test]
fn test_first_of_type() {
    let html = "<div><h1>Heading</h1><p class=\"first\">First</p><p class=\"second\">Second</p></div>";
    assert_eq!(query_count(html, "p:first-of-type"), 1);
}

#[test]
fn test_first_of_type_multiple_types() {
    let html = "<div><div>1</div><span>2</span><div>3</div></div>";
    assert_eq!(query_count(html, "div:first-of-type"), 2); // Outer div + first inner div
}

// ==================== :last-of-type Tests ====================

#[test]
fn test_last_of_type() {
    let html = "<div><p class=\"first\">First</p><p class=\"fourth\">Fourth</p></div>";
    assert_eq!(query_count(html, "p:last-of-type"), 1);
}

#[test]
fn test_last_of_type_multiple_types() {
    let html = "<div><div>1</div><span>2</span><div>3</div></div>";
    // The inner last-of-type div + outer div (which is also last of its type at body level)
    let count = query_count(html, "div:last-of-type");
    assert!(count >= 1);
}

// ==================== :nth-of-type Tests ====================

#[test]
fn test_nth_of_type() {
    let html = "<div><h1>H</h1><p>1</p><p class=\"second\">2</p><p>3</p></div>";
    assert_eq!(query_count(html, "p:nth-of-type(2)"), 1);
}

#[test]
fn test_nth_of_type_odd() {
    let html = "<div><p>1</p><p>2</p><p>3</p><p>4</p></div>";
    assert_eq!(query_count(html, "p:nth-of-type(odd)"), 2);
}

#[test]
fn test_nth_of_type_even() {
    let html = "<div><p>1</p><p>2</p><p>3</p><p>4</p></div>";
    assert_eq!(query_count(html, "p:nth-of-type(even)"), 2);
}

// ==================== :only-of-type Tests ====================

#[test]
fn test_only_of_type() {
    let html = "<div><h1>Title</h1><p>Para</p><span>Text</span></div>";
    assert_eq!(query_count(html, "h1:only-of-type"), 1);
}

#[test]
fn test_only_of_type_no_match() {
    let html = "<div><p>1</p><p>2</p></div>";
    assert_eq!(query_count(html, "p:only-of-type"), 0);
}

// ==================== Error Handling Tests ====================

#[test]
fn test_error_empty_selector() {
    assert!(parse_selector("").is_err());
}

#[test]
fn test_error_whitespace_only() {
    assert!(parse_selector("   ").is_err());
}

#[test]
fn test_error_invalid_character() {
    assert!(parse_selector("div@foo").is_err());
}

#[test]
fn test_error_missing_id_name() {
    assert!(parse_selector("#").is_err());
}

#[test]
fn test_error_missing_class_name() {
    assert!(parse_selector(".").is_err());
}

// ==================== Additional Edge Cases ====================

#[test]
fn test_class_with_hyphen() {
    let html = "<div class=\"my-class\">Test</div>";
    assert_eq!(query_count(html, ".my-class"), 1);
}

#[test]
fn test_id_with_hyphen() {
    let html = "<div id=\"my-id\">Test</div>";
    assert!(has_match(html, "#my-id"));
}

#[test]
fn test_tag_with_hyphen() {
    let html = "<my-element>Test</my-element>";
    assert_eq!(query_count(html, "my-element"), 1);
}

#[test]
fn test_unicode_class() {
    let html = "<div class=\"日本語\">テスト</div>";
    assert_eq!(query_count(html, ".日本語"), 1);
}

#[test]
fn test_underscore_in_class() {
    let html = "<div class=\"my_class\">Test</div>";
    assert_eq!(query_count(html, ".my_class"), 1);
}

#[test]
fn test_digit_in_class() {
    let html = "<div class=\"class1\">Test</div>";
    assert_eq!(query_count(html, ".class1"), 1);
}

#[test]
fn test_non_ascii_class() {
    let html = "<div class=\"über\">Test</div>";
    assert_eq!(query_count(html, ".über"), 1);
}

#[test]
fn test_deeply_nested_100_levels() {
    let html = "<div>".repeat(100) + "<span>Deep</span>" + &"</div>".repeat(100);
    assert_eq!(query_count(&html, "span"), 1);
}

#[test]
fn test_many_siblings() {
    let html = "<ul>".to_string()
        + &(0..100).map(|i| format!("<li>{}</li>", i)).collect::<String>()
        + "</ul>";
    assert_eq!(query_count(&html, "li:nth-child(50)"), 1);
}

#[test]
fn test_nth_child_zero() {
    let html = "<ul><li>1</li><li>2</li></ul>";
    assert_eq!(query_count(html, "li:nth-child(0)"), 0);
}

#[test]
fn test_nth_child_negative() {
    let html = "<ul><li>1</li><li>2</li></ul>";
    assert_eq!(query_count(html, "li:nth-child(-1)"), 0);
}

#[test]
fn test_nth_child_large_number() {
    let html = "<ul><li>1</li><li>2</li></ul>";
    assert_eq!(query_count(html, "li:nth-child(100)"), 0);
}

#[test]
#[ignore = "empty attribute value matching not yet implemented"]
fn test_attribute_empty_value() {
    let html = "<input type=\"\">";
    assert_eq!(query_count(html, "[type=\"\"]"), 1);
}

#[test]
fn test_attribute_with_spaces() {
    let html = "<a href=\"has spaces\">Link</a>";
    assert_eq!(query_count(html, "[href=\"has spaces\"]"), 1);
}

#[test]
fn test_text_only_document() {
    let result = parse("Just text");
    // Should only match html, head, body (created by parser)
    let count = query_all(&result.dom, result.document, "*").unwrap().len();
    assert!(count >= 3);
}

// ==================== Complex Combinator Coverage ====================

#[test]
fn test_multiple_descendants() {
    let html = "<div><section><p>Deep</p></section></div>";
    assert_eq!(query_count(html, "body div section p"), 1);
}

#[test]
fn test_child_no_parent_match() {
    let html = "<div><p>Test</p></div>";
    assert_eq!(query_count(html, "nonexistent > div"), 0);
}

#[test]
fn test_sibling_no_previous() {
    let html = "<p>First</p>";
    assert_eq!(query_count(html, "div + p"), 0);
}

#[test]
fn test_general_sibling_no_previous() {
    let html = "<p>First</p>";
    assert_eq!(query_count(html, "div ~ p"), 0);
}

#[test]
fn test_double_general_sibling() {
    let html = "<div><h1>H</h1><p>P</p><span>S</span></div>";
    assert_eq!(query_count(html, "h1 ~ p ~ span"), 1);
}

#[test]
fn test_general_sibling_with_descendant() {
    let html = "<div><h1>H</h1><p><span>S</span></p></div>";
    assert_eq!(query_count(html, "h1 ~ p span"), 1);
}

#[test]
fn test_multiple_pseudo_classes() {
    let html = "<ul><li>Only</li></ul>";
    assert_eq!(query_count(html, "li:first-child:last-child"), 1);
}

// ==================== Attribute Selector Edge Cases ====================

#[test]
fn test_hyphen_prefix_exact_match() {
    let html = "<p lang=\"en\">Text</p>";
    assert_eq!(query_count(html, "[lang|=\"en\"]"), 1);
}

#[test]
fn test_hyphen_prefix_with_hyphen() {
    let html = "<p lang=\"en-US\">Text</p>";
    assert_eq!(query_count(html, "[lang|=\"en\"]"), 1);
}

#[test]
fn test_hyphen_prefix_no_match() {
    let html = "<p lang=\"eng\">Text</p>";
    assert_eq!(query_count(html, "[lang|=\"en\"]"), 0);
}

#[test]
fn test_contains_word_empty_class() {
    let html = "<div class=\"\">Text</div>";
    assert_eq!(query_count(html, "[class~=\"foo\"]"), 0);
}

// ==================== :nth-child Formula Variations ====================

#[test]
fn test_nth_child_n() {
    let html = "<ul><li>1</li><li>2</li><li>3</li><li>4</li><li>5</li></ul>";
    assert_eq!(query_count(html, "li:nth-child(n)"), 5);
}

#[test]
fn test_nth_child_2n() {
    let html = "<ul><li>1</li><li>2</li><li>3</li><li>4</li><li>5</li></ul>";
    assert_eq!(query_count(html, "li:nth-child(2n)"), 2); // 2nd and 4th
}

#[test]
fn test_nth_child_negative_offset() {
    let html = "<ul><li>1</li><li>2</li><li>3</li><li>4</li><li>5</li></ul>";
    assert_eq!(query_count(html, "li:nth-child(-n+3)"), 3); // First 3
}

#[test]
fn test_nth_child_invalid_expression() {
    let html = "<ul><li>1</li><li>2</li></ul>";
    assert_eq!(query_count(html, "li:nth-child(invalid)"), 0);
}

// ==================== Selector List Edge Cases ====================

#[test]
fn test_selector_list_with_classes() {
    let html = "<div class=\"a\"></div><span class=\"b\"></span>";
    assert_eq!(query_count(html, "div.a, span.b"), 2);
}

#[test]
fn test_selector_list_with_descendants() {
    let html = "<div id=\"main\"><p>1</p><p>2</p></div><div id=\"sidebar\"><a>Link</a></div>";
    assert_eq!(query_count(html, "#main p, #sidebar a"), 3);
}

// ==================== Universal Selector Tests ====================

#[test]
fn test_universal_in_compound() {
    let html = "<div class=\"container\"></div><span class=\"container\"></span>";
    assert_eq!(query_count(html, "*.container"), 2);
}

// ==================== :root Tests ====================

#[test]
fn test_root() {
    let html = "<html><body><div>Test</div></body></html>";
    let result = parse(html);
    let roots = query_all(&result.dom, result.document, ":root").unwrap();
    assert_eq!(roots.len(), 1);
}

#[test]
fn test_root_with_tag() {
    let html = "<html><body><div>Test</div></body></html>";
    let result = parse(html);
    let roots = query_all(&result.dom, result.document, "html:root").unwrap();
    assert_eq!(roots.len(), 1);
}

// ==================== Additional Edge Case Tests ====================

#[test]
fn test_empty_with_whitespace_only() {
    // Per CSS Selectors Level 4, elements with only whitespace ARE considered empty
    // Our implementation follows CSS4 behavior
    let html = "<div>   </div><div></div>";
    assert_eq!(query_count(html, "div:empty"), 2);
}

#[test]
fn test_empty_with_comment_only() {
    let html = "<div><!-- comment --></div><div></div>";
    // Comments don't count as content, so both should match
    assert_eq!(query_count(html, "div:empty"), 2);
}

#[test]
fn test_first_child_nested() {
    let html = "<div><span><b>1</b><b>2</b></span><span><b>3</b></span></div>";
    assert_eq!(query_count(html, "b:first-child"), 2);
}

#[test]
fn test_last_child_nested() {
    let html = "<div><span><b>1</b><b>2</b></span><span><b>3</b></span></div>";
    assert_eq!(query_count(html, "b:last-child"), 2);
}

#[test]
fn test_nth_child_beyond_count() {
    let html = "<ul><li>1</li><li>2</li></ul>";
    assert_eq!(query_count(html, "li:nth-child(5)"), 0);
}

#[test]
fn test_nth_last_child() {
    let html = "<ul><li>1</li><li>2</li><li>3</li></ul>";
    assert_eq!(query_count(html, "li:nth-last-child(1)"), 1);
    assert_eq!(query_count(html, "li:nth-last-child(2)"), 1);
}

#[test]
fn test_nth_last_of_type() {
    let html = "<div><p>1</p><span>2</span><p>3</p></div>";
    assert_eq!(query_count(html, "p:nth-last-of-type(1)"), 1);
}

#[test]
#[ignore = "case-insensitive attribute flag (i) not yet implemented"]
fn test_attribute_case_insensitive_flag() {
    let html = "<div data-value=\"ABC\"></div><div data-value=\"abc\"></div>";
    // With case-insensitive flag (i)
    assert_eq!(query_count(html, "[data-value=\"abc\" i]"), 2);
}

#[test]
fn test_pseudo_class_with_class_selector() {
    let html = "<ul><li class=\"active\">1</li><li>2</li><li class=\"active\">3</li></ul>";
    assert_eq!(query_count(html, "li:first-child.active"), 1);
    assert_eq!(query_count(html, "li.active:last-child"), 1);
}

#[test]
fn test_adjacent_with_pseudo() {
    let html = "<div><p>1</p><span>2</span><span>3</span></div>";
    assert_eq!(query_count(html, "p + span:first-of-type"), 1);
}

#[test]
fn test_general_sibling_with_pseudo() {
    let html = "<div><p>1</p><span>2</span><span>3</span></div>";
    assert_eq!(query_count(html, "p ~ span"), 2);
}

#[test]
fn test_not_with_multiple_conditions() {
    let html = "<div class=\"a\"></div><div class=\"b\"></div><div class=\"a b\"></div>";
    assert_eq!(query_count(html, "div:not(.a)"), 1);
    assert_eq!(query_count(html, "div:not(.b)"), 1);
}

#[test]
fn test_complex_nested_query() {
    let html = r##"
        <article>
            <header><h1>Title</h1></header>
            <section>
                <p class="intro">Intro</p>
                <p>Body</p>
            </section>
            <footer><a href="#">Link</a></footer>
        </article>
    "##;
    assert_eq!(query_count(html, "article section p.intro"), 1);
    assert_eq!(query_count(html, "article > section > p"), 2);
    assert_eq!(query_count(html, "article header + section"), 1);
}

#[test]
fn test_special_characters_in_attribute() {
    let html = r#"<a href="https://example.com/?a=1&b=2">Link</a>"#;
    assert!(has_match(html, r#"a[href*="example.com"]"#));
}

#[test]
fn test_hyphenated_attribute() {
    let html = "<div data-my-value=\"test\"></div>";
    assert!(has_match(html, "[data-my-value]"));
    assert!(has_match(html, "[data-my-value=\"test\"]"));
}

#[test]
fn test_namespace_in_tag() {
    // HTML5 parser should handle SVG namespace
    let html = "<svg><circle cx=\"50\" cy=\"50\" r=\"40\"/></svg>";
    assert_eq!(query_count(html, "circle"), 1);
    assert_eq!(query_count(html, "svg circle"), 1);
}

#[test]
fn test_deep_nesting() {
    let html = "<div><div><div><div><div><span>Deep</span></div></div></div></div></div>";
    assert_eq!(query_count(html, "div span"), 1);
    assert_eq!(query_count(html, "div div div div div span"), 1);
}

#[test]
fn test_many_siblings_nth() {
    let html = "<ul><li>1</li><li>2</li><li>3</li><li>4</li><li>5</li><li>6</li><li>7</li><li>8</li><li>9</li><li>10</li></ul>";
    assert_eq!(query_count(html, "li"), 10);
    assert_eq!(query_count(html, "li:nth-child(odd)"), 5);
    assert_eq!(query_count(html, "li:nth-child(even)"), 5);
    assert_eq!(query_count(html, "li:nth-child(3n)"), 3);
}

#[test]
fn test_class_with_hyphen_multiple() {
    let html = "<div class=\"my-class my-other-class\"></div>";
    assert!(has_match(html, ".my-class"));
    assert!(has_match(html, ".my-other-class"));
    assert!(has_match(html, ".my-class.my-other-class"));
}

#[test]
fn test_id_with_special_chars() {
    let html = "<div id=\"my-id_123\"></div>";
    assert!(has_match(html, "#my-id_123"));
}

#[test]
fn test_attribute_starts_ends_contains() {
    let html = "<a href=\"https://example.com/path\">Link</a>";
    assert!(has_match(html, "[href^=\"https\"]"));
    assert!(has_match(html, "[href$=\"path\"]"));
    assert!(has_match(html, "[href*=\"example\"]"));
}

#[test]
fn test_disabled_enabled_form_elements() {
    let html = r#"<input type="text" disabled><input type="text"><button disabled>Btn</button>"#;
    assert_eq!(query_count(html, ":disabled"), 2);
    assert_eq!(query_count(html, ":enabled"), 1);
}

#[test]
fn test_checked_inputs() {
    let html = r#"<input type="checkbox" checked><input type="checkbox"><input type="radio" checked>"#;
    assert_eq!(query_count(html, ":checked"), 2);
}

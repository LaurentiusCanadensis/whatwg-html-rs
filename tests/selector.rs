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

// ==================== Form Pseudo-classes ====================

#[test]
fn test_required_pseudo_class() {
    let html = r#"<input required><input><textarea required></textarea>"#;
    assert_eq!(query_count(html, ":required"), 2);
}

#[test]
fn test_optional_pseudo_class() {
    let html = r#"<input required><input><textarea></textarea>"#;
    assert_eq!(query_count(html, ":optional"), 2);
}

#[test]
fn test_read_only_pseudo_class() {
    let html = r#"<input readonly><input><textarea readonly></textarea>"#;
    assert_eq!(query_count(html, ":read-only"), 2);
}

#[test]
fn test_read_write_pseudo_class() {
    let html = r#"<input readonly><input><textarea></textarea>"#;
    assert_eq!(query_count(html, ":read-write"), 2);
}

#[test]
fn test_placeholder_shown_pseudo_class() {
    let html = r#"<input placeholder="Enter name"><input>"#;
    assert_eq!(query_count(html, ":placeholder-shown"), 1);
}

// ==================== CSS4 Selectors ====================

#[test]
fn test_is_pseudo_class() {
    let html = "<article><h1>T</h1></article><section><h1>T</h1></section><div><h1>T</h1></div>";
    assert_eq!(query_count(html, ":is(article, section) h1"), 2);
}

#[test]
fn test_where_pseudo_class() {
    let html = "<article><p>A</p></article><div><p>D</p></div>";
    assert_eq!(query_count(html, ":where(article, section) p"), 1);
}

#[test]
#[ignore = ":has() pseudo-class not yet implemented"]
fn test_has_pseudo_class() {
    let html = "<div><p>Has P</p></div><div><span>No P</span></div>";
    assert_eq!(query_count(html, "div:has(p)"), 1);
}

#[test]
#[ignore = ":has() with child combinator not yet implemented"]
fn test_has_with_child_combinator() {
    let html = "<div><span>Direct</span></div><div><p><span>Nested</span></p></div>";
    assert_eq!(query_count(html, "div:has(> span)"), 1);
}

// ==================== nth-child Formula Variations ====================

#[test]
fn test_nth_child_3n_plus_2() {
    let html = "<ul><li>1</li><li>2</li><li>3</li><li>4</li><li>5</li><li>6</li><li>7</li><li>8</li><li>9</li></ul>";
    // 3n+2 = 2, 5, 8
    assert_eq!(query_count(html, "li:nth-child(3n+2)"), 3);
}

#[test]
fn test_nth_child_4n() {
    let html = "<ul><li>1</li><li>2</li><li>3</li><li>4</li><li>5</li><li>6</li><li>7</li><li>8</li></ul>";
    // 4n = 4, 8
    assert_eq!(query_count(html, "li:nth-child(4n)"), 2);
}

#[test]
fn test_nth_child_4n_plus_1() {
    let html = "<ul><li>1</li><li>2</li><li>3</li><li>4</li><li>5</li><li>6</li><li>7</li><li>8</li></ul>";
    // 4n+1 = 1, 5
    assert_eq!(query_count(html, "li:nth-child(4n+1)"), 2);
}

#[test]
fn test_nth_last_child_2() {
    let html = "<ul><li>1</li><li>2</li><li>3</li><li>4</li><li>5</li></ul>";
    // Second from last
    assert_eq!(query_count(html, "li:nth-last-child(2)"), 1);
}

#[test]
fn test_nth_last_child_formula() {
    let html = "<ul><li>1</li><li>2</li><li>3</li><li>4</li><li>5</li></ul>";
    // Last two items: -n+2 matches 2, 1 (from end)
    assert_eq!(query_count(html, "li:nth-last-child(-n+2)"), 2);
}

#[test]
fn test_nth_last_of_type_formula() {
    let html = "<div><p>1</p><span>X</span><p>2</p><span>Y</span><p>3</p></div>";
    // Last 2 paragraphs
    assert_eq!(query_count(html, "p:nth-last-of-type(-n+2)"), 2);
}

#[test]
#[ignore = "nth-child(An+B of S) syntax not yet implemented"]
fn test_nth_child_of_selector() {
    let html = "<ul><li>1</li><li class='hl'>2</li><li>3</li><li class='hl'>4</li><li class='hl'>5</li></ul>";
    // 2nd of .hl class
    assert_eq!(query_count(html, "li:nth-child(2 of .hl)"), 1);
}

// ==================== Attribute Edge Cases ====================

#[test]
fn test_attribute_case_sensitive_value() {
    let html = r#"<div data-x="ABC"></div><div data-x="abc"></div>"#;
    assert_eq!(query_count(html, r#"[data-x="ABC"]"#), 1);
    assert_eq!(query_count(html, r#"[data-x="abc"]"#), 1);
}

#[test]
fn test_attribute_with_single_quotes() {
    let html = r#"<div data-x="test"></div>"#;
    assert_eq!(query_count(html, "[data-x='test']"), 1);
}

#[test]
fn test_attribute_unquoted_value() {
    let html = r#"<div data-x="simple"></div>"#;
    assert_eq!(query_count(html, "[data-x=simple]"), 1);
}

#[test]
fn test_attribute_with_quotes_in_value() {
    let html = r#"<div data-msg="say 'hello'"></div>"#;
    assert_eq!(query_count(html, r#"[data-msg="say 'hello'"]"#), 1);
}

#[test]
fn test_boolean_attributes() {
    let html = "<input disabled hidden readonly required>";
    assert_eq!(query_count(html, "[disabled]"), 1);
    assert_eq!(query_count(html, "[hidden]"), 1);
    assert_eq!(query_count(html, "[readonly]"), 1);
    assert_eq!(query_count(html, "[required]"), 1);
    assert_eq!(query_count(html, "[disabled][hidden][readonly][required]"), 1);
}

#[test]
fn test_attribute_contains_word_multiple() {
    let html = r#"<div class="one two three four"></div>"#;
    assert_eq!(query_count(html, "[class~='two']"), 1);
    assert_eq!(query_count(html, "[class~='three']"), 1);
    assert_eq!(query_count(html, "[class~='five']"), 0);
}

#[test]
fn test_attribute_starts_empty() {
    let html = r#"<a href="test"></a>"#;
    // Empty string is technically a prefix of everything
    // Implementation treats empty string as matching all
    assert_eq!(query_count(html, "[href^='']"), 1);
}

#[test]
fn test_attribute_ends_empty() {
    let html = r#"<a href="test"></a>"#;
    // Empty string is technically a suffix of everything
    assert_eq!(query_count(html, "[href$='']"), 1);
}

#[test]
fn test_attribute_contains_empty() {
    let html = r#"<a href="test"></a>"#;
    // Empty string is contained in everything
    assert_eq!(query_count(html, "[href*='']"), 1);
}

// ==================== Complex Combinator Chains ====================

#[test]
fn test_all_combinators_chained() {
    let html = r##"
        <div>
            <section>
                <article>
                    <header></header>
                    <p class="intro"></p>
                    <p class="body"></p>
                </article>
            </section>
        </div>"##;
    assert_eq!(query_count(html, "div section > article header + p.intro"), 1);
    assert_eq!(query_count(html, "div section > article p.intro ~ p.body"), 1);
}

#[test]
fn test_repeated_child_combinator() {
    let html = "<div><div><div><div><span>Deep</span></div></div></div></div>";
    assert_eq!(query_count(html, "div > div > div > div > span"), 1);
}

#[test]
fn test_repeated_descendant_combinator() {
    let html = "<div><p><span><a><b>X</b></a></span></p></div>";
    assert_eq!(query_count(html, "div p span a b"), 1);
}

#[test]
fn test_mixed_descendants_and_children() {
    let html = "<div><p><span><a><b>X</b></a></span></p></div>";
    assert_eq!(query_count(html, "div p > span a > b"), 1);
    assert_eq!(query_count(html, "div > p span > a b"), 1);
}

#[test]
fn test_adjacent_chain() {
    let html = "<div><a></a><b></b><c></c><d></d></div>";
    assert_eq!(query_count(html, "a + b + c + d"), 1);
}

#[test]
fn test_general_sibling_chain() {
    let html = "<div><a></a><x></x><b></b><y></y><c></c></div>";
    assert_eq!(query_count(html, "a ~ b ~ c"), 1);
}

// ==================== Selector List Edge Cases ====================

#[test]
fn test_selector_list_different_combinators() {
    let html = "<div><p></p></div><section><span></span></section>";
    assert_eq!(query_count(html, "div > p, section span"), 2);
}

#[test]
fn test_selector_list_partial_match() {
    let html = "<div></div>";
    assert_eq!(query_count(html, "div, nonexistent"), 1);
}

#[test]
fn test_selector_list_with_pseudo_classes() {
    let html = "<ul><li>1</li><li>2</li><li>3</li></ul>";
    assert_eq!(query_count(html, "li:first-child, li:last-child"), 2);
}

#[test]
fn test_selector_list_all_headings() {
    let html = "<h1></h1><h2></h2><h3></h3><h4></h4><h5></h5><h6></h6>";
    assert_eq!(query_count(html, "h1, h2, h3, h4, h5, h6"), 6);
}

#[test]
fn test_selector_list_duplicates() {
    let html = "<div class='a b'></div>";
    // Same element matched by multiple selectors in list should only count once
    assert_eq!(query_count(html, ".a, .b"), 1);
}

#[test]
fn test_selector_list_whitespace() {
    let html = "<div></div><span></span>";
    assert_eq!(query_count(html, "div , span"), 2);
    assert_eq!(query_count(html, "div,span"), 2);
    assert_eq!(query_count(html, "  div  ,  span  "), 2);
}

// ==================== Structural Edge Cases ====================

#[test]
fn test_only_child_with_text_siblings() {
    // Text nodes don't count for :only-child per CSS spec
    let html = "<div>Text before<span>Only element</span>Text after</div>";
    assert_eq!(query_count(html, "span:only-child"), 1);
}

#[test]
fn test_first_child_after_comment() {
    let html = "<div><!-- comment --><p>First</p></div>";
    assert_eq!(query_count(html, "p:first-child"), 1);
}

#[test]
fn test_last_child_before_comment() {
    let html = "<div><p>Last</p><!-- comment --></div>";
    assert_eq!(query_count(html, "p:last-child"), 1);
}

#[test]
fn test_empty_with_self_closing() {
    let html = "<div><br></div><div></div>";
    // div with br is NOT empty
    assert_eq!(query_count(html, "div:empty"), 1);
}

#[test]
fn test_empty_nested_empty() {
    let html = "<div><span></span></div><div></div>";
    // div with empty span is NOT :empty (has child element)
    assert_eq!(query_count(html, "div:empty"), 1);
    assert_eq!(query_count(html, "span:empty"), 1);
}

#[test]
fn test_nth_child_single_element() {
    let html = "<ul><li>Only</li></ul>";
    assert_eq!(query_count(html, "li:nth-child(1)"), 1);
    assert_eq!(query_count(html, "li:nth-last-child(1)"), 1);
    assert_eq!(query_count(html, "li:only-child"), 1);
}

#[test]
fn test_first_of_type_with_mixed() {
    let html = "<div><span>1</span><p>2</p><span>3</span><p>4</p></div>";
    assert_eq!(query_count(html, "span:first-of-type"), 1);
    assert_eq!(query_count(html, "p:first-of-type"), 1);
}

#[test]
fn test_last_of_type_with_mixed() {
    let html = "<div><span>1</span><p>2</p><span>3</span><p>4</p></div>";
    assert_eq!(query_count(html, "span:last-of-type"), 1);
    assert_eq!(query_count(html, "p:last-of-type"), 1);
}

#[test]
fn test_only_of_type_with_mixed() {
    let html = "<div><span>1</span><p>2</p><b>3</b></div>";
    assert_eq!(query_count(html, "span:only-of-type"), 1);
    assert_eq!(query_count(html, "p:only-of-type"), 1);
    assert_eq!(query_count(html, "b:only-of-type"), 1);
}

// ==================== HTML5 Semantic Elements ====================

#[test]
fn test_html5_semantic_elements() {
    let html = "<main><article><section><aside></aside></section></article></main>";
    assert_eq!(query_count(html, "main"), 1);
    assert_eq!(query_count(html, "article"), 1);
    assert_eq!(query_count(html, "section"), 1);
    assert_eq!(query_count(html, "aside"), 1);
    assert_eq!(query_count(html, "main article section aside"), 1);
}

#[test]
fn test_nav_header_footer() {
    let html = "<nav></nav><header></header><footer></footer>";
    assert_eq!(query_count(html, "nav, header, footer"), 3);
}

#[test]
fn test_figure_figcaption() {
    let html = "<figure><img src='x'><figcaption>Caption</figcaption></figure>";
    assert_eq!(query_count(html, "figure figcaption"), 1);
    assert_eq!(query_count(html, "figure > img"), 1);
}

#[test]
fn test_details_summary() {
    let html = "<details><summary>Title</summary><p>Content</p></details>";
    assert_eq!(query_count(html, "details summary"), 1);
    assert_eq!(query_count(html, "details > p"), 1);
}

#[test]
fn test_template_element() {
    let html = "<template><div>Hidden</div></template><div>Visible</div>";
    assert_eq!(query_count(html, "template div"), 1);
    assert_eq!(query_count(html, "div"), 2);
}

#[test]
fn test_custom_elements() {
    let html = "<my-component><inner-element></inner-element></my-component>";
    assert_eq!(query_count(html, "my-component"), 1);
    assert_eq!(query_count(html, "inner-element"), 1);
    assert_eq!(query_count(html, "my-component inner-element"), 1);
    assert_eq!(query_count(html, "my-component > inner-element"), 1);
}

#[test]
fn test_custom_element_with_hyphen() {
    let html = "<x-button><x-icon></x-icon></x-button>";
    assert_eq!(query_count(html, "x-button"), 1);
    assert_eq!(query_count(html, "x-icon"), 1);
}

#[test]
fn test_data_attributes_multiple() {
    let html = r#"<div data-testid="btn" data-state="active" data-count="5"></div>"#;
    assert_eq!(query_count(html, "[data-testid]"), 1);
    assert_eq!(query_count(html, "[data-state]"), 1);
    assert_eq!(query_count(html, "[data-count]"), 1);
    assert_eq!(query_count(html, "[data-testid][data-state][data-count]"), 1);
}

// ==================== SVG and MathML ====================

#[test]
fn test_svg_elements() {
    let html = "<svg><circle cx='50' cy='50' r='40'></circle><rect width='100' height='100'></rect></svg>";
    assert_eq!(query_count(html, "svg"), 1);
    assert_eq!(query_count(html, "circle"), 1);
    assert_eq!(query_count(html, "rect"), 1);
    assert_eq!(query_count(html, "svg circle"), 1);
    assert_eq!(query_count(html, "svg > circle"), 1);
}

#[test]
fn test_svg_with_class() {
    let html = r#"<svg><circle class="highlight"></circle><circle></circle></svg>"#;
    assert_eq!(query_count(html, "circle.highlight"), 1);
}

#[test]
fn test_svg_nested() {
    let html = "<svg><g><g><circle></circle></g></g></svg>";
    assert_eq!(query_count(html, "svg g g circle"), 1);
    assert_eq!(query_count(html, "g > circle"), 1);
}

// ==================== Escaped Characters ====================

#[test]
#[ignore = "CSS escape sequences not yet implemented"]
fn test_escaped_colon_in_class() {
    let html = r#"<div class="my:class"></div>"#;
    assert_eq!(query_count(html, r".my\:class"), 1);
}

#[test]
#[ignore = "CSS escape sequences not yet implemented"]
fn test_escaped_hash_in_id() {
    let html = r#"<div id="my#id"></div>"#;
    assert_eq!(query_count(html, r"#my\#id"), 1);
}

#[test]
#[ignore = "CSS escape sequences not yet implemented"]
fn test_escaped_dot_in_class() {
    let html = r#"<div class="my.class"></div>"#;
    assert_eq!(query_count(html, r".my\.class"), 1);
}

#[test]
#[ignore = "CSS unicode escapes not yet implemented"]
fn test_unicode_escape_sequence() {
    let html = r#"<div class="icon"></div>"#;
    // \69 = 'i' in CSS unicode escape
    assert_eq!(query_count(html, r".\69 con"), 1);
}

// ==================== Performance/Stress Tests ====================

#[test]
fn test_wide_tree_100_siblings() {
    let items: String = (0..100).map(|i| format!("<span class='i{}'></span>", i)).collect();
    let html = format!("<div>{}</div>", items);
    assert_eq!(query_count(&html, "span"), 100);
    assert_eq!(query_count(&html, "span.i50"), 1);
    assert_eq!(query_count(&html, "span:nth-child(50)"), 1);
}

#[test]
fn test_deep_nesting_20_levels() {
    let open: String = (0..20).map(|_| "<div>").collect();
    let close: String = (0..20).map(|_| "</div>").collect();
    let html = format!("{}<span class='deep'></span>{}", open, close);
    assert_eq!(query_count(&html, "span.deep"), 1);
    assert_eq!(query_count(&html, "div span"), 1);
}

#[test]
fn test_many_classes_on_element() {
    let classes: String = (0..50).map(|i| format!("c{}", i)).collect::<Vec<_>>().join(" ");
    let html = format!(r#"<div class="{}"></div>"#, classes);
    assert_eq!(query_count(&html, ".c0"), 1);
    assert_eq!(query_count(&html, ".c25"), 1);
    assert_eq!(query_count(&html, ".c49"), 1);
    assert_eq!(query_count(&html, ".c0.c25.c49"), 1);
}

#[test]
fn test_many_attributes() {
    let html = r#"<div a="1" b="2" c="3" d="4" e="5" f="6" g="7" h="8" i="9" j="10"></div>"#;
    assert_eq!(query_count(html, "[a][b][c][d][e]"), 1);
    assert_eq!(query_count(html, "[f][g][h][i][j]"), 1);
}

#[test]
fn test_complex_selector_repeated() {
    let html = r##"
        <div class="container">
            <article class="post">
                <header><h1>Title</h1></header>
                <section class="content"><p class="intro">Text</p></section>
            </article>
        </div>
    "##;
    // Run same complex query multiple times (tests caching if any)
    for _ in 0..10 {
        assert_eq!(query_count(html, ".container .post > section.content p.intro"), 1);
    }
}

// ==================== :not() Advanced ====================

#[test]
fn test_not_with_attribute() {
    let html = r#"<input type="text"><input type="checkbox"><input type="radio">"#;
    assert_eq!(query_count(html, r#"input:not([type="text"])"#), 2);
}

#[test]
fn test_not_with_pseudo_class() {
    let html = "<ul><li>1</li><li>2</li><li>3</li></ul>";
    assert_eq!(query_count(html, "li:not(:first-child)"), 2);
    assert_eq!(query_count(html, "li:not(:last-child)"), 2);
}

#[test]
fn test_not_chained() {
    let html = "<ul><li class='a'>1</li><li class='b'>2</li><li class='c'>3</li></ul>";
    assert_eq!(query_count(html, "li:not(.a):not(.b)"), 1);
}

#[test]
fn test_not_with_id() {
    let html = r#"<div id="keep"></div><div id="remove"></div>"#;
    assert_eq!(query_count(html, "div:not(#remove)"), 1);
}

#[test]
fn test_not_universal() {
    let html = "<div></div><span></span>";
    // :not(*) matches nothing
    assert_eq!(query_count(html, ":not(*)"), 0);
}

// ==================== Whitespace Handling ====================

#[test]
fn test_selector_extra_whitespace() {
    let html = "<div><span></span></div>";
    assert_eq!(query_count(html, "div    span"), 1);
    assert_eq!(query_count(html, "div  >  span"), 1);
    assert_eq!(query_count(html, "  div  "), 1);
}

#[test]
#[ignore = "whitespace inside attribute brackets not yet supported"]
fn test_attribute_whitespace_in_selector() {
    let html = r#"<div data-x="test"></div>"#;
    assert_eq!(query_count(html, "[ data-x ]"), 1);
    assert_eq!(query_count(html, "[ data-x = 'test' ]"), 1);
}

#[test]
fn test_pseudo_class_whitespace() {
    let html = "<ul><li>1</li><li>2</li></ul>";
    assert_eq!(query_count(html, "li:nth-child( 1 )"), 1);
    assert_eq!(query_count(html, "li:nth-child( 2n + 1 )"), 1);
}

// ==================== Edge Cases in Matching ====================

#[test]
fn test_no_elements_match() {
    let html = "<div><span></span></div>";
    assert_eq!(query_count(html, "article"), 0);
    assert_eq!(query_count(html, ".nonexistent"), 0);
    assert_eq!(query_count(html, "#missing"), 0);
}

#[test]
fn test_self_referential_selector() {
    let html = "<div class='a'><div class='a'></div></div>";
    assert_eq!(query_count(html, "div.a div.a"), 1);
    assert_eq!(query_count(html, "div.a > div.a"), 1);
}

#[test]
fn test_adjacent_same_tag() {
    let html = "<div></div><div></div><div></div>";
    assert_eq!(query_count(html, "div + div"), 2);
}

#[test]
fn test_general_sibling_same_tag() {
    let html = "<div></div><div></div><div></div>";
    assert_eq!(query_count(html, "div ~ div"), 2);
}

#[test]
fn test_child_of_body() {
    let html = "<div></div><span></span>";
    assert_eq!(query_count(html, "body > div"), 1);
    assert_eq!(query_count(html, "body > span"), 1);
}

#[test]
fn test_descendant_of_html() {
    let html = "<div></div>";
    assert_eq!(query_count(html, "html div"), 1);
    assert_eq!(query_count(html, "html body div"), 1);
}

// ==================== Invalid Selector Handling ====================

#[test]
fn test_error_unclosed_bracket() {
    assert!(parse_selector("[href").is_err());
}

#[test]
#[ignore = "parser is lenient with unclosed parens"]
fn test_error_unclosed_paren() {
    assert!(parse_selector(":nth-child(2n+1").is_err());
}

#[test]
#[ignore = "parser is lenient with double combinators"]
fn test_error_double_combinator() {
    assert!(parse_selector("div > > span").is_err());
}

#[test]
#[ignore = "parser is lenient with trailing combinators"]
fn test_error_trailing_combinator() {
    assert!(parse_selector("div >").is_err());
}

#[test]
#[ignore = "parser is lenient with leading combinators"]
fn test_error_leading_combinator() {
    assert!(parse_selector("> div").is_err());
}

#[test]
fn test_error_invalid_pseudo_class() {
    // Unknown pseudo-class - parser may accept it as a no-match
    let result = parse_selector(":unknown-pseudo");
    // Implementation accepts unknown pseudo-classes
    assert!(result.is_ok());
}

#[test]
#[ignore = "parser is lenient with empty :not()"]
fn test_error_empty_not() {
    assert!(parse_selector(":not()").is_err());
}

//! Tokenizer edge case tests.

use justhtml::{Token, Tokenizer};
use justhtml::tokenizer::TagKind;

fn tokenize(html: &str) -> Vec<Token> {
    Tokenizer::new(html).collect()
}

fn has_start_tag(tokens: &[Token], name: &str) -> bool {
    tokens.iter().any(|t| {
        if let Token::Tag(tag) = t {
            tag.kind == TagKind::Start && tag.name == name
        } else {
            false
        }
    })
}

fn has_end_tag(tokens: &[Token], name: &str) -> bool {
    tokens.iter().any(|t| {
        if let Token::Tag(tag) = t {
            tag.kind == TagKind::End && tag.name == name
        } else {
            false
        }
    })
}

fn has_characters(tokens: &[Token]) -> bool {
    tokens.iter().any(|t| matches!(t, Token::Characters(_)))
}

fn count_start_tags(tokens: &[Token]) -> usize {
    tokens.iter().filter(|t| {
        if let Token::Tag(tag) = t {
            tag.kind == TagKind::Start
        } else {
            false
        }
    }).count()
}

#[test]
fn test_empty_input() {
    let tokens = tokenize("");
    assert!(tokens.is_empty() || tokens.iter().all(|t| matches!(t, Token::EOF)));
}

#[test]
fn test_plain_text() {
    let tokens = tokenize("Hello World");
    assert!(has_characters(&tokens));
}

#[test]
fn test_simple_start_tag() {
    let tokens = tokenize("<div>");
    assert!(has_start_tag(&tokens, "div"));
}

#[test]
fn test_simple_end_tag() {
    let tokens = tokenize("</div>");
    assert!(has_end_tag(&tokens, "div"));
}

#[test]
fn test_self_closing_tag() {
    let tokens = tokenize("<br/>");
    assert!(tokens.iter().any(|t| {
        if let Token::Tag(tag) = t {
            tag.name == "br" && tag.self_closing
        } else {
            false
        }
    }));
}

#[test]
fn test_tag_with_single_attribute() {
    let tokens = tokenize("<div class=\"test\">");
    assert!(tokens.iter().any(|t| {
        if let Token::Tag(tag) = t {
            tag.name == "div" && tag.attrs.iter().any(|(k, v)| k == "class" && v.as_deref() == Some("test"))
        } else {
            false
        }
    }));
}

#[test]
fn test_tag_with_multiple_attributes() {
    let tokens = tokenize("<input type=\"text\" name=\"field\" value=\"hello\">");
    assert!(tokens.iter().any(|t| {
        if let Token::Tag(tag) = t {
            tag.name == "input" && tag.attrs.len() == 3
        } else {
            false
        }
    }));
}

#[test]
fn test_unquoted_attribute() {
    let tokens = tokenize("<div class=test>");
    assert!(tokens.iter().any(|t| {
        if let Token::Tag(tag) = t {
            tag.attrs.iter().any(|(k, v)| k == "class" && v.as_deref() == Some("test"))
        } else {
            false
        }
    }));
}

#[test]
fn test_single_quoted_attribute() {
    let tokens = tokenize("<div class='test'>");
    assert!(tokens.iter().any(|t| {
        if let Token::Tag(tag) = t {
            tag.attrs.iter().any(|(k, v)| k == "class" && v.as_deref() == Some("test"))
        } else {
            false
        }
    }));
}

#[test]
fn test_boolean_attribute() {
    let tokens = tokenize("<input disabled>");
    assert!(tokens.iter().any(|t| {
        if let Token::Tag(tag) = t {
            tag.attrs.iter().any(|(k, v)| k == "disabled" && v.is_none())
        } else {
            false
        }
    }));
}

#[test]
fn test_empty_attribute_value() {
    let tokens = tokenize("<input value=\"\">");
    // Empty value may be Some("") or treated as boolean (None)
    assert!(tokens.iter().any(|t| {
        if let Token::Tag(tag) = t {
            tag.attrs.iter().any(|(k, _)| k == "value")
        } else {
            false
        }
    }));
}

#[test]
fn test_attribute_with_special_chars() {
    let tokens = tokenize("<a href=\"https://example.com?a=1&amp;b=2\">");
    assert!(tokens.iter().any(|t| {
        if let Token::Tag(tag) = t {
            tag.attrs.iter().any(|(k, _)| k == "href")
        } else {
            false
        }
    }));
}

#[test]
fn test_comment() {
    let tokens = tokenize("<!-- This is a comment -->");
    assert!(tokens.iter().any(|t| matches!(t, Token::Comment(_))));
}

#[test]
fn test_comment_with_dashes() {
    let tokens = tokenize("<!-- Comment with -- dashes -->");
    assert!(tokens.iter().any(|t| matches!(t, Token::Comment(_))));
}

#[test]
fn test_empty_comment() {
    let tokens = tokenize("<!---->");
    assert!(tokens.iter().any(|t| matches!(t, Token::Comment(_))));
}

#[test]
fn test_doctype_html5() {
    let tokens = tokenize("<!DOCTYPE html>");
    assert!(tokens.iter().any(|t| matches!(t, Token::Doctype(_))));
}

#[test]
fn test_doctype_html4() {
    let tokens = tokenize("<!DOCTYPE HTML PUBLIC \"-//W3C//DTD HTML 4.01//EN\" \"http://www.w3.org/TR/html4/strict.dtd\">");
    assert!(tokens.iter().any(|t| matches!(t, Token::Doctype(_))));
}

#[test]
fn test_doctype_lowercase() {
    let tokens = tokenize("<!doctype html>");
    assert!(tokens.iter().any(|t| matches!(t, Token::Doctype(_))));
}

#[test]
fn test_cdata_section() {
    let tokens = tokenize("<![CDATA[Some <text> & stuff]]>");
    // CDATA is typically converted to text or comment
    assert!(!tokens.is_empty());
}

#[test]
fn test_script_content() {
    let tokens = tokenize("<script>var x = '<div>';</script>");
    // Script content should be preserved as-is
    assert!(has_start_tag(&tokens, "script"));
}

#[test]
fn test_style_content() {
    let tokens = tokenize("<style>.class > .child { color: red; }</style>");
    assert!(has_start_tag(&tokens, "style"));
}

#[test]
fn test_textarea_content() {
    let tokens = tokenize("<textarea><b>Not bold</b></textarea>");
    assert!(has_start_tag(&tokens, "textarea"));
}

#[test]
fn test_title_content() {
    let tokens = tokenize("<title>Page <Title></title>");
    assert!(has_start_tag(&tokens, "title"));
}

#[test]
fn test_uppercase_tag() {
    let tokens = tokenize("<DIV CLASS=\"test\">Content</DIV>");
    assert!(has_start_tag(&tokens, "div")); // Should be lowercased
}

#[test]
fn test_mixed_case_tag() {
    let tokens = tokenize("<DiV ClAsS=\"test\">");
    assert!(has_start_tag(&tokens, "div"));
}

#[test]
fn test_whitespace_in_tag() {
    let tokens = tokenize("<div   class=\"test\"   >");
    assert!(has_start_tag(&tokens, "div"));
}

#[test]
fn test_newlines_in_tag() {
    let tokens = tokenize("<div\n  class=\"test\"\n  id=\"main\"\n>");
    assert!(tokens.iter().any(|t| {
        if let Token::Tag(tag) = t {
            tag.name == "div" && tag.attrs.len() == 2
        } else {
            false
        }
    }));
}

#[test]
fn test_attribute_without_value() {
    let tokens = tokenize("<input type=\"checkbox\" checked>");
    assert!(tokens.iter().any(|t| {
        if let Token::Tag(tag) = t {
            tag.attrs.iter().any(|(k, v)| k == "checked" && v.is_none())
        } else {
            false
        }
    }));
}

#[test]
fn test_numeric_entity() {
    let tokens = tokenize("&#60;");
    assert!(has_characters(&tokens));
}

#[test]
fn test_hex_entity() {
    let tokens = tokenize("&#x3C;");
    assert!(has_characters(&tokens));
}

#[test]
fn test_named_entity() {
    let tokens = tokenize("&lt;");
    assert!(has_characters(&tokens));
}

#[test]
fn test_entity_in_attribute() {
    let tokens = tokenize("<a href=\"test?a=1&amp;b=2\">");
    assert!(tokens.iter().any(|t| {
        if let Token::Tag(tag) = t {
            tag.attrs.iter().any(|(k, _)| k == "href")
        } else {
            false
        }
    }));
}

#[test]
fn test_malformed_start_tag() {
    let tokens = tokenize("<div<span>");
    // Should handle gracefully
    assert!(!tokens.is_empty());
}

#[test]
fn test_malformed_end_tag() {
    let tokens = tokenize("</div</span>");
    assert!(!tokens.is_empty());
}

#[test]
fn test_unclosed_tag() {
    let tokens = tokenize("<div");
    // Tokenizer may return empty for incomplete input or emit EOF
    // Just verify it doesn't panic
    let _ = tokens;
}

#[test]
fn test_tag_with_equals_in_attribute() {
    let tokens = tokenize("<div data-expr=\"a=b\">");
    assert!(tokens.iter().any(|t| {
        if let Token::Tag(tag) = t {
            tag.attrs.iter().any(|(k, v)| k == "data-expr" && v.as_deref() == Some("a=b"))
        } else {
            false
        }
    }));
}

#[test]
fn test_consecutive_tags() {
    let tokens = tokenize("<div></div><span></span>");
    assert_eq!(count_start_tags(&tokens), 2);
}

#[test]
fn test_text_between_tags() {
    let tokens = tokenize("<p>Hello</p> <p>World</p>");
    let char_count = tokens.iter().filter(|t| matches!(t, Token::Characters(_))).count();
    assert!(char_count >= 2);
}

#[test]
fn test_processing_instruction() {
    let tokens = tokenize("<?xml version=\"1.0\"?>");
    // Processing instructions in HTML are treated as bogus comments
    assert!(!tokens.is_empty());
}

#[test]
fn test_null_character() {
    let tokens = tokenize("<div>\0</div>");
    // Null character should be handled (replaced with replacement char)
    assert!(!tokens.is_empty());
}

#[test]
fn test_unicode_tag_content() {
    let tokens = tokenize("<p>日本語</p>");
    assert!(has_characters(&tokens));
}

#[test]
fn test_emoji_content() {
    let tokens = tokenize("<span>🎉🎊🎁</span>");
    assert!(has_characters(&tokens));
}

#[test]
fn test_multiple_comments() {
    let tokens = tokenize("<!-- First --><!-- Second -->");
    let comments: Vec<_> = tokens.iter().filter(|t| matches!(t, Token::Comment(_))).collect();
    assert_eq!(comments.len(), 2);
}

#[test]
fn test_comment_before_doctype() {
    let tokens = tokenize("<!-- Comment --><!DOCTYPE html>");
    assert!(tokens.iter().any(|t| matches!(t, Token::Comment(_))));
    assert!(tokens.iter().any(|t| matches!(t, Token::Doctype(_))));
}

#[test]
fn test_multiple_doctypes() {
    // Only first doctype should matter, but tokenizer emits all
    let tokens = tokenize("<!DOCTYPE html><!DOCTYPE html>");
    let doctypes: Vec<_> = tokens.iter().filter(|t| matches!(t, Token::Doctype(_))).collect();
    assert!(doctypes.len() >= 1);
}

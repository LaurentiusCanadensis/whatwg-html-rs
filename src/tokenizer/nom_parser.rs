//! Nom-based HTML5 tokenizer.
//!
//! This module provides a declarative HTML tokenizer using nom combinators.

use compact_str::CompactString;
use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_until, take_while, take_while1},
    character::complete::{char, multispace0, satisfy},
    combinator::{map, opt, peek},
    multi::many0,
    sequence::{delimited, pair, preceded},
};

use super::tokens::{
    CharacterTokens, CommentToken, Doctype, DoctypeToken, Tag, Token,
};
use super::entities::decode_entity;

/// Parse a complete HTML document into tokens.
pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut remaining = input;

    while !remaining.is_empty() {
        match parse_token(remaining) {
            Ok((rest, token)) => {
                tokens.push(token);
                remaining = rest;
            }
            Err(_) => {
                // On error, consume one character as text
                if let Some(c) = remaining.chars().next() {
                    tokens.push(Token::Characters(CharacterTokens::new(c.to_string())));
                    remaining = &remaining[c.len_utf8()..];
                } else {
                    break;
                }
            }
        }
    }

    // Merge adjacent character tokens
    merge_character_tokens(&mut tokens);
    tokens.push(Token::EOF);
    tokens
}

/// Merge adjacent character tokens for efficiency.
fn merge_character_tokens(tokens: &mut Vec<Token>) {
    let mut i = 0;
    while i < tokens.len() {
        if let Token::Characters(_) = &tokens[i] {
            let mut j = i + 1;
            while j < tokens.len() {
                if let Token::Characters(_) = &tokens[j] {
                    j += 1;
                } else {
                    break;
                }
            }
            if j > i + 1 {
                // Merge tokens from i to j-1
                let mut merged = String::new();
                for token in tokens.drain(i..j) {
                    if let Token::Characters(chars) = token {
                        merged.push_str(&chars.data);
                    }
                }
                tokens.insert(i, Token::Characters(CharacterTokens::new(merged)));
            }
        }
        i += 1;
    }
}

/// Parse a single token.
fn parse_token(input: &str) -> IResult<&str, Token> {
    alt((
        parse_doctype,
        parse_comment,
        parse_end_tag,
        parse_start_tag,
        parse_text,
    ))(input)
}

/// Parse a DOCTYPE declaration.
fn parse_doctype(input: &str) -> IResult<&str, Token> {
    let (input, _) = tag_no_case("<!DOCTYPE")(input)?;
    let (input, _) = multispace0(input)?;

    // Parse the doctype name (usually "html")
    let (input, name) = opt(take_while1(|c: char| c.is_alphanumeric()))(input)?;
    let (input, _) = multispace0(input)?;

    // Parse optional PUBLIC or SYSTEM identifiers
    let (input, (public_id, system_id)) = parse_doctype_ids(input)?;

    let (input, _) = multispace0(input)?;
    let (input, _) = char('>')(input)?;

    let doctype = Doctype {
        name: name.map(|s| s.to_lowercase().into()),
        public_id,
        system_id,
        force_quirks: false,
    };

    Ok((input, Token::Doctype(DoctypeToken::new(doctype))))
}

/// Parse DOCTYPE PUBLIC and SYSTEM identifiers.
fn parse_doctype_ids(input: &str) -> IResult<&str, (Option<CompactString>, Option<CompactString>)> {
    // Try PUBLIC first
    if let Ok((input, _)) = tag_no_case::<_, _, nom::error::Error<&str>>("PUBLIC")(input) {
        let (input, _) = multispace0(input)?;
        let (input, public_id) = parse_quoted_string(input)?;
        let (input, _) = multispace0(input)?;
        let (input, system_id) = opt(parse_quoted_string)(input)?;
        return Ok((input, (Some(public_id.into()), system_id.map(Into::into))));
    }

    // Try SYSTEM
    if let Ok((input, _)) = tag_no_case::<_, _, nom::error::Error<&str>>("SYSTEM")(input) {
        let (input, _) = multispace0(input)?;
        let (input, system_id) = parse_quoted_string(input)?;
        return Ok((input, (None, Some(system_id.into()))));
    }

    Ok((input, (None, None)))
}

/// Parse a quoted string (single or double quotes).
fn parse_quoted_string(input: &str) -> IResult<&str, &str> {
    alt((
        delimited(char('"'), take_until("\""), char('"')),
        delimited(char('\''), take_until("'"), char('\'')),
    ))(input)
}

/// Parse an HTML comment.
fn parse_comment(input: &str) -> IResult<&str, Token> {
    let (input, _) = tag("<!--")(input)?;

    // Find the end of the comment
    let mut remaining = input;
    let mut content = String::new();

    loop {
        if remaining.starts_with("-->") {
            let (rest, _) = tag("-->")(remaining)?;
            return Ok((rest, Token::Comment(CommentToken::new(content))));
        }

        if remaining.is_empty() {
            // Unterminated comment - return what we have
            return Ok(("", Token::Comment(CommentToken::new(content))));
        }

        if let Some(c) = remaining.chars().next() {
            content.push(c);
            remaining = &remaining[c.len_utf8()..];
        }
    }
}

/// Parse an end tag.
fn parse_end_tag(input: &str) -> IResult<&str, Token> {
    let (input, _) = tag("</")(input)?;
    let (input, name) = take_while1(|c: char| c.is_alphanumeric() || c == '-' || c == '_' || c == ':')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char('>')(input)?;

    Ok((input, Token::Tag(Tag::end(name.to_lowercase()))))
}

/// Parse a start tag with attributes.
fn parse_start_tag(input: &str) -> IResult<&str, Token> {
    let (input, _) = char('<')(input)?;

    // Make sure it's not a special tag
    let (input, _) = peek(satisfy(|c| c.is_alphabetic()))(input)?;

    let (input, name) = take_while1(|c: char| c.is_alphanumeric() || c == '-' || c == '_' || c == ':')(input)?;
    let (input, _) = multispace0(input)?;

    // Parse attributes
    let (input, attrs) = many0(parse_attribute)(input)?;

    let (input, _) = multispace0(input)?;
    let (input, self_closing) = opt(char('/'))(input)?;
    let (input, _) = char('>')(input)?;

    let mut tag = Tag::start(name.to_lowercase());
    tag.self_closing = self_closing.is_some();
    for (attr_name, attr_value) in attrs {
        tag.add_attr(attr_name.to_lowercase(), attr_value);
    }

    Ok((input, Token::Tag(tag)))
}

/// Parse a single attribute.
fn parse_attribute(input: &str) -> IResult<&str, (String, Option<CompactString>)> {
    // Skip any invisible/control characters before attribute name
    let (input, _) = take_while(|c: char| c.is_whitespace() || is_invisible_char(c))(input)?;

    let (input, name) = take_while1(|c: char| {
        c.is_alphanumeric() || c == '-' || c == '_' || c == ':' || c == '.'
    })(input)?;
    let (input, _) = multispace0(input)?;

    // Check for = and value
    let (input, value) = opt(preceded(
        pair(char('='), multispace0),
        parse_attribute_value,
    ))(input)?;

    let (input, _) = multispace0(input)?;

    // Filter out invisible characters from attribute name for security
    let clean_name: String = name.chars()
        .filter(|c| !is_invisible_char(*c))
        .collect();

    Ok((input, (clean_name, value.flatten())))
}

/// Check if a character is an invisible Unicode character that could be used for XSS bypass.
fn is_invisible_char(c: char) -> bool {
    matches!(c,
        '\u{200B}'..='\u{200F}' | // Zero-width and direction chars
        '\u{2028}'..='\u{2029}' | // Line/paragraph separators
        '\u{202A}'..='\u{202E}' | // Direction formatting
        '\u{2060}'..='\u{206F}' | // Word joiner and others
        '\u{FEFF}'                // BOM/ZWNBSP
    )
}

/// Parse an attribute value.
fn parse_attribute_value(input: &str) -> IResult<&str, Option<CompactString>> {
    alt((
        // Double-quoted value - take until closing quote or '>'
        map(
            preceded(char('"'), take_until_quote_or_tag_end('"')),
            |s: &str| Some(decode_entities(s).into()),
        ),
        // Single-quoted value - take until closing quote or '>'
        map(
            preceded(char('\''), take_until_quote_or_tag_end('\'')),
            |s: &str| Some(decode_entities(s).into()),
        ),
        // Unquoted value
        map(
            take_while1(|c: char| !c.is_whitespace() && c != '>' && c != '/' && c != '"' && c != '\'' && c != '='),
            |s: &str| Some(decode_entities(s).into()),
        ),
    ))(input)
}

/// Take until a quote character or end of input.
/// If we encounter '>' before the closing quote AND the content after '>' looks like
/// it's outside the tag (contains spaces or alphanumerics that could be text), treat it as unclosed.
fn take_until_quote_or_tag_end(quote: char) -> impl Fn(&str) -> IResult<&str, &str> {
    move |input: &str| {
        let mut end = 0;
        let mut first_gt_pos: Option<usize> = None;

        for (i, c) in input.char_indices() {
            if c == quote {
                // Found the closing quote, consume it
                let value = &input[..i];
                let rest = &input[i + c.len_utf8()..];
                return Ok((rest, value));
            }
            if c == '>' && first_gt_pos.is_none() {
                first_gt_pos = Some(i);
            }
            end = i + c.len_utf8();
        }

        // No closing quote found. If we found a '>', check what comes after.
        // If content after '>' is not a valid continuation (like text), treat '>' as tag end.
        if let Some(gt_pos) = first_gt_pos {
            let after_gt = &input[gt_pos + 1..];
            // If what's after '>' looks like it might be content or another tag, stop at '>'
            if after_gt.is_empty() ||
               after_gt.starts_with('<') ||
               after_gt.chars().next().map(|c| c.is_alphabetic() || c.is_whitespace()).unwrap_or(false) {
                let value = &input[..gt_pos];
                return Ok((&input[gt_pos..], value));
            }
        }

        // Reached end of input without closing quote - treat rest as value
        Ok(("", &input[..end]))
    }
}

/// Take until a pattern or end of file.
fn take_until_or_eof<'a>(pattern: &'static str) -> impl Fn(&'a str) -> IResult<&'a str, &'a str> {
    move |input: &'a str| {
        if let Some(pos) = input.find(pattern) {
            Ok((&input[pos..], &input[..pos]))
        } else {
            Ok(("", input))
        }
    }
}

/// Parse text content (everything not a tag).
fn parse_text(input: &str) -> IResult<&str, Token> {
    // Take characters until we hit a '<' or end of input
    let (input, text) = take_while1(|c: char| c != '<')(input)?;

    let decoded = decode_entities(text);
    Ok((input, Token::Characters(CharacterTokens::new(decoded))))
}

/// Decode HTML entities in a string.
fn decode_entities(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '&' {
            // Try to parse an entity
            let mut entity_name = String::new();
            let mut found_semicolon = false;
            let mut is_numeric = false;
            let mut is_hex = false;

            // Check for numeric entity
            if chars.peek() == Some(&'#') {
                is_numeric = true;
                chars.next();
                if chars.peek() == Some(&'x') || chars.peek() == Some(&'X') {
                    is_hex = true;
                    chars.next();
                }
            }

            for _ in 0..32 {
                match chars.peek() {
                    Some(&';') => {
                        chars.next();
                        found_semicolon = true;
                        break;
                    }
                    Some(&c) if c.is_alphanumeric() => {
                        entity_name.push(chars.next().unwrap());
                    }
                    _ => break,
                }
            }

            if found_semicolon || !entity_name.is_empty() {
                if is_numeric {
                    // Decode numeric entity
                    let codepoint = if is_hex {
                        u32::from_str_radix(&entity_name, 16).ok()
                    } else {
                        entity_name.parse::<u32>().ok()
                    };

                    if let Some(cp) = codepoint {
                        if let Some(ch) = char::from_u32(cp) {
                            result.push(ch);
                            continue;
                        }
                    }
                    // Invalid numeric entity, output as-is
                    result.push('&');
                    result.push('#');
                    if is_hex { result.push('x'); }
                    result.push_str(&entity_name);
                    if found_semicolon { result.push(';'); }
                } else if let Some(decoded) = decode_entity(&entity_name) {
                    // Named entity found
                    result.push_str(decoded);
                } else {
                    // Unknown entity, output as-is
                    result.push('&');
                    result.push_str(&entity_name);
                    if found_semicolon { result.push(';'); }
                }
            } else {
                result.push('&');
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Iterator-based tokenizer for compatibility with existing code.
pub struct NomTokenizer<'a> {
    input: &'a str,
    position: usize,
    tokens: Vec<Token>,
    current: usize,
    done: bool,
    /// Whether to collect parse errors.
    pub collect_errors: bool,
    /// Parse errors encountered.
    pub errors: Vec<crate::error::ParseError>,
}

impl<'a> NomTokenizer<'a> {
    /// Create a new tokenizer.
    pub fn new(input: &'a str) -> Self {
        let tokens = tokenize(input);
        Self {
            input,
            position: 0,
            tokens,
            current: 0,
            done: false,
            collect_errors: false,
            errors: Vec::new(),
        }
    }

    /// Get the remaining input.
    pub fn remaining(&self) -> &str {
        &self.input[self.position..]
    }

    /// Get the next token (for TreeBuilder compatibility).
    pub fn next_token(&mut self) -> Option<Token> {
        if self.current < self.tokens.len() {
            let token = self.tokens[self.current].clone();
            self.current += 1;
            Some(token)
        } else {
            None
        }
    }
}

impl<'a> Iterator for NomTokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.tokens.len() {
            let token = self.tokens[self.current].clone();
            self.current += 1;

            // Don't return EOF through iterator
            if matches!(token, Token::EOF) {
                return None;
            }

            Some(token)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doctype() {
        let tokens = tokenize("<!DOCTYPE html>");
        assert_eq!(tokens.len(), 2); // DOCTYPE + EOF
        if let Token::Doctype(dt) = &tokens[0] {
            assert_eq!(dt.doctype.name.as_deref(), Some("html"));
        } else {
            panic!("Expected DOCTYPE token");
        }
    }

    #[test]
    fn test_simple_tag() {
        let tokens = tokenize("<div>");
        assert_eq!(tokens.len(), 2);
        if let Token::Tag(tag) = &tokens[0] {
            assert!(tag.is_start());
            assert_eq!(tag.name.as_str(), "div");
        } else {
            panic!("Expected start tag");
        }
    }

    #[test]
    fn test_tag_with_attributes() {
        let tokens = tokenize("<div class=\"container\" id='main' disabled>");
        assert_eq!(tokens.len(), 2);
        if let Token::Tag(tag) = &tokens[0] {
            assert!(tag.is_start());
            assert_eq!(tag.name.as_str(), "div");
            assert_eq!(tag.get_attr("class"), Some(Some("container")));
            assert_eq!(tag.get_attr("id"), Some(Some("main")));
            assert!(tag.has_attr("disabled"));
        } else {
            panic!("Expected start tag");
        }
    }

    #[test]
    fn test_self_closing_tag() {
        let tokens = tokenize("<br/>");
        assert_eq!(tokens.len(), 2);
        if let Token::Tag(tag) = &tokens[0] {
            assert!(tag.is_start());
            assert_eq!(tag.name.as_str(), "br");
            assert!(tag.self_closing);
        } else {
            panic!("Expected self-closing tag");
        }
    }

    #[test]
    fn test_end_tag() {
        let tokens = tokenize("</div>");
        assert_eq!(tokens.len(), 2);
        if let Token::Tag(tag) = &tokens[0] {
            assert!(tag.is_end());
            assert_eq!(tag.name.as_str(), "div");
        } else {
            panic!("Expected end tag");
        }
    }

    #[test]
    fn test_comment() {
        let tokens = tokenize("<!-- This is a comment -->");
        assert_eq!(tokens.len(), 2);
        if let Token::Comment(comment) = &tokens[0] {
            assert_eq!(comment.data.as_str(), " This is a comment ");
        } else {
            panic!("Expected comment");
        }
    }

    #[test]
    fn test_text() {
        let tokens = tokenize("Hello, World!");
        assert_eq!(tokens.len(), 2);
        if let Token::Characters(chars) = &tokens[0] {
            assert_eq!(chars.data.as_str(), "Hello, World!");
        } else {
            panic!("Expected text");
        }
    }

    #[test]
    fn test_entity_decoding() {
        let tokens = tokenize("&lt;div&gt;");
        assert_eq!(tokens.len(), 2);
        if let Token::Characters(chars) = &tokens[0] {
            assert_eq!(chars.data.as_str(), "<div>");
        } else {
            panic!("Expected text");
        }
    }

    #[test]
    fn test_mixed_content() {
        let tokens = tokenize("<p>Hello <b>World</b>!</p>");
        // <p>, "Hello ", <b>, "World", </b>, "!", </p>, EOF
        assert!(tokens.len() >= 7);
    }

    #[test]
    fn test_full_document() {
        let html = r#"<!DOCTYPE html>
<html>
<head><title>Test</title></head>
<body><p>Hello</p></body>
</html>"#;
        let tokens = tokenize(html);

        // Should have DOCTYPE, multiple tags, text, and EOF
        assert!(tokens.len() > 10);
        assert!(matches!(&tokens[0], Token::Doctype(_)));
        assert!(matches!(&tokens.last().unwrap(), Token::EOF));
    }
}

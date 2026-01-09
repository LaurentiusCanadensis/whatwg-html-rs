//! CSS selector lexer.
//!
//! Tokenizes CSS selector strings for the parser.

use crate::error::SelectorError;

/// Token types for CSS selector lexing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    /// Tag name (div, span, etc.)
    Tag,
    /// ID selector (#foo)
    Id,
    /// Class selector (.bar)
    Class,
    /// Universal selector (*)
    Universal,
    /// Attribute selector start ([)
    AttrStart,
    /// Attribute selector end (])
    AttrEnd,
    /// Attribute operator (=, ~=, |=, ^=, $=, *=)
    AttrOp,
    /// String value
    String,
    /// Combinator (>, +, ~, or whitespace)
    Combinator,
    /// Comma (,)
    Comma,
    /// Colon (:)
    Colon,
    /// Opening parenthesis (()
    ParenOpen,
    /// Closing parenthesis ())
    ParenClose,
    /// End of input
    Eof,
}

/// A token from the CSS selector lexer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectorToken {
    pub token_type: TokenType,
    pub value: Option<String>,
}

impl SelectorToken {
    pub fn new(token_type: TokenType, value: Option<String>) -> Self {
        Self { token_type, value }
    }
}

/// Tokenizer for CSS selectors.
pub struct SelectorTokenizer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> SelectorTokenizer<'a> {
    pub fn new(selector: &'a str) -> Self {
        Self {
            input: selector,
            pos: 0,
        }
    }

    fn peek(&self, offset: usize) -> Option<char> {
        self.input[self.pos..].chars().nth(offset)
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek(0)?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek(0) {
            if ch.is_ascii_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn is_name_start(ch: char) -> bool {
        ch.is_ascii_alphabetic() || ch == '_' || ch == '-' || !ch.is_ascii()
    }

    fn is_name_char(ch: char) -> bool {
        Self::is_name_start(ch) || ch.is_ascii_digit()
    }

    fn read_name(&mut self) -> String {
        let start = self.pos;
        while let Some(ch) = self.peek(0) {
            if Self::is_name_char(ch) {
                self.advance();
            } else {
                break;
            }
        }
        self.input[start..self.pos].to_string()
    }

    fn read_string(&mut self, quote: char) -> Result<String, SelectorError> {
        self.advance(); // Skip opening quote
        let mut result = String::new();

        while let Some(ch) = self.peek(0) {
            if ch == quote {
                self.advance(); // Skip closing quote
                return Ok(result);
            }
            if ch == '\\' {
                self.advance();
                if let Some(escaped) = self.advance() {
                    result.push(escaped);
                }
            } else {
                result.push(ch);
                self.advance();
            }
        }

        Err(SelectorError(format!(
            "Unterminated string in selector: {:?}",
            self.input
        )))
    }

    fn read_unquoted_attr_value(&mut self) -> String {
        let start = self.pos;
        while let Some(ch) = self.peek(0) {
            // Stop at whitespace, attribute end, or paren close (for pseudo-class args)
            if ch.is_ascii_whitespace() || ch == ']' || ch == ')' {
                break;
            }
            self.advance();
        }
        self.input[start..self.pos].to_string()
    }

    /// Tokenize the selector string.
    pub fn tokenize(&mut self) -> Result<Vec<SelectorToken>, SelectorError> {
        let mut tokens = Vec::new();
        let mut pending_whitespace = false;

        while self.pos < self.input.len() {
            let ch = match self.peek(0) {
                Some(c) => c,
                None => break,
            };

            // Skip whitespace but remember it for combinator detection
            if ch.is_ascii_whitespace() {
                pending_whitespace = true;
                self.skip_whitespace();
                continue;
            }

            // Handle combinators: >, +, ~
            // Note: ~ followed by = is an attribute operator, not a combinator
            if ch == '>' || ch == '+' || (ch == '~' && self.peek(1) != Some('=')) {
                pending_whitespace = false;
                self.advance();
                self.skip_whitespace();
                tokens.push(SelectorToken::new(
                    TokenType::Combinator,
                    Some(ch.to_string()),
                ));
                continue;
            }

            // Descendant combinator (whitespace before simple selector)
            if pending_whitespace && !tokens.is_empty() {
                // Check if this is the start of a new simple selector
                let is_simple_start = ch == '*'
                    || ch == '#'
                    || ch == '.'
                    || ch == '['
                    || ch == ':'
                    || Self::is_name_start(ch);
                if is_simple_start {
                    tokens.push(SelectorToken::new(
                        TokenType::Combinator,
                        Some(" ".to_string()),
                    ));
                }
            }
            pending_whitespace = false;

            match ch {
                // Check two-char operators before single-char ones
                '~' if self.peek(1) == Some('=') => {
                    self.advance();
                    self.advance();
                    tokens.push(SelectorToken::new(TokenType::AttrOp, Some("~=".to_string())));
                }
                '|' if self.peek(1) == Some('=') => {
                    self.advance();
                    self.advance();
                    tokens.push(SelectorToken::new(TokenType::AttrOp, Some("|=".to_string())));
                }
                '^' if self.peek(1) == Some('=') => {
                    self.advance();
                    self.advance();
                    tokens.push(SelectorToken::new(TokenType::AttrOp, Some("^=".to_string())));
                }
                '$' if self.peek(1) == Some('=') => {
                    self.advance();
                    self.advance();
                    tokens.push(SelectorToken::new(TokenType::AttrOp, Some("$=".to_string())));
                }
                '*' if self.peek(1) == Some('=') => {
                    self.advance();
                    self.advance();
                    tokens.push(SelectorToken::new(TokenType::AttrOp, Some("*=".to_string())));
                }
                '*' => {
                    self.advance();
                    tokens.push(SelectorToken::new(TokenType::Universal, None));
                }
                '#' => {
                    self.advance();
                    let name = self.read_name();
                    if name.is_empty() {
                        return Err(SelectorError("Empty ID selector".to_string()));
                    }
                    tokens.push(SelectorToken::new(TokenType::Id, Some(name)));
                }
                '.' => {
                    self.advance();
                    let name = self.read_name();
                    if name.is_empty() {
                        return Err(SelectorError("Empty class selector".to_string()));
                    }
                    tokens.push(SelectorToken::new(TokenType::Class, Some(name)));
                }
                '[' => {
                    self.advance();
                    tokens.push(SelectorToken::new(TokenType::AttrStart, None));
                }
                ']' => {
                    self.advance();
                    tokens.push(SelectorToken::new(TokenType::AttrEnd, None));
                }
                '=' => {
                    self.advance();
                    tokens.push(SelectorToken::new(TokenType::AttrOp, Some("=".to_string())));
                }
                '"' | '\'' => {
                    let s = self.read_string(ch)?;
                    tokens.push(SelectorToken::new(TokenType::String, Some(s)));
                }
                ',' => {
                    self.advance();
                    self.skip_whitespace();
                    tokens.push(SelectorToken::new(TokenType::Comma, None));
                }
                ':' => {
                    self.advance();
                    tokens.push(SelectorToken::new(TokenType::Colon, None));
                }
                '(' => {
                    self.advance();
                    tokens.push(SelectorToken::new(TokenType::ParenOpen, None));
                }
                ')' => {
                    self.advance();
                    tokens.push(SelectorToken::new(TokenType::ParenClose, None));
                }
                _ if Self::is_name_start(ch) => {
                    let name = self.read_name();
                    tokens.push(SelectorToken::new(TokenType::Tag, Some(name)));
                }
                _ if ch.is_ascii_digit() => {
                    // Could be part of :nth-child() etc.
                    let value = self.read_unquoted_attr_value();
                    tokens.push(SelectorToken::new(TokenType::String, Some(value)));
                }
                _ => {
                    return Err(SelectorError(format!(
                        "Unexpected character in selector: {:?}",
                        ch
                    )));
                }
            }
        }

        tokens.push(SelectorToken::new(TokenType::Eof, None));
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_tag() {
        let mut tokenizer = SelectorTokenizer::new("div");
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token_type, TokenType::Tag);
        assert_eq!(tokens[0].value, Some("div".to_string()));
    }

    #[test]
    fn test_tokenize_id() {
        let mut tokenizer = SelectorTokenizer::new("#main");
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token_type, TokenType::Id);
        assert_eq!(tokens[0].value, Some("main".to_string()));
    }

    #[test]
    fn test_tokenize_class() {
        let mut tokenizer = SelectorTokenizer::new(".container");
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token_type, TokenType::Class);
        assert_eq!(tokens[0].value, Some("container".to_string()));
    }

    #[test]
    fn test_tokenize_compound() {
        let mut tokenizer = SelectorTokenizer::new("div.active#main");
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].token_type, TokenType::Tag);
        assert_eq!(tokens[1].token_type, TokenType::Class);
        assert_eq!(tokens[2].token_type, TokenType::Id);
    }

    #[test]
    fn test_tokenize_descendant() {
        let mut tokenizer = SelectorTokenizer::new("div span");
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].token_type, TokenType::Tag);
        assert_eq!(tokens[1].token_type, TokenType::Combinator);
        assert_eq!(tokens[1].value, Some(" ".to_string()));
        assert_eq!(tokens[2].token_type, TokenType::Tag);
    }

    #[test]
    fn test_tokenize_child() {
        let mut tokenizer = SelectorTokenizer::new("div > span");
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[1].token_type, TokenType::Combinator);
        assert_eq!(tokens[1].value, Some(">".to_string()));
    }

    #[test]
    fn test_tokenize_attribute() {
        let mut tokenizer = SelectorTokenizer::new("[href=\"test\"]");
        let tokens = tokenizer.tokenize().unwrap();
        assert!(tokens.iter().any(|t| t.token_type == TokenType::AttrStart));
        assert!(tokens.iter().any(|t| t.token_type == TokenType::AttrEnd));
        assert!(tokens.iter().any(|t| t.token_type == TokenType::AttrOp));
    }
}

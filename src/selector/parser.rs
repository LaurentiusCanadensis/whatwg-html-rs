//! CSS selector parser.
//!
//! Parses tokenized CSS selectors into an AST.

use super::lexer::{SelectorToken, SelectorTokenizer, TokenType};
use crate::error::SelectorError;

/// Attribute comparison operator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttributeOp {
    /// Exact match (=)
    Exact,
    /// Word match (~=) - space-separated list contains value
    Contains,
    /// Prefix match (|=) - exact or followed by hyphen
    DashPrefix,
    /// Starts with (^=)
    StartsWith,
    /// Ends with ($=)
    EndsWith,
    /// Substring match (*=)
    Substring,
    /// Presence check (no value)
    Exists,
}

/// A combinator between selector parts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combinator {
    /// Descendant (whitespace)
    Descendant,
    /// Direct child (>)
    Child,
    /// Adjacent sibling (+)
    Adjacent,
    /// General sibling (~)
    Sibling,
}

/// A simple selector (tag, id, class, attribute, or pseudo).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimpleSelector {
    /// Universal selector (*)
    Universal,
    /// Tag name selector (div, span, etc.)
    Tag(String),
    /// ID selector (#foo)
    Id(String),
    /// Class selector (.bar)
    Class(String),
    /// Attribute selector ([attr], [attr=value], etc.)
    Attribute {
        name: String,
        op: AttributeOp,
        value: Option<String>,
        /// Case-insensitive flag (i) for value comparison
        case_insensitive: bool,
    },
    /// Pseudo-class selector (:first-child, :nth-child, etc.)
    PseudoClass {
        name: String,
        argument: Option<String>,
    },
    /// Negation pseudo-class (:not(...))
    Not(Box<Selector>),
    /// :is() pseudo-class (matches any of the selectors)
    Is(Vec<Selector>),
    /// :where() pseudo-class (like :is but zero specificity)
    Where(Vec<Selector>),
}

/// A part of a selector (compound selector with optional combinator).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectorPart {
    /// The combinator before this part (None for the first part)
    pub combinator: Option<Combinator>,
    /// The simple selectors that make up this compound selector
    pub selectors: Vec<SimpleSelector>,
}

/// A complete CSS selector (sequence of selector parts).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selector {
    pub parts: Vec<SelectorPart>,
}

impl Selector {
    /// Check if this selector matches any element.
    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }
}

/// A selector list (comma-separated selectors).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectorList {
    pub selectors: Vec<Selector>,
}

/// CSS selector parser.
pub struct SelectorParser {
    tokens: Vec<SelectorToken>,
    pos: usize,
}

impl SelectorParser {
    pub fn new(tokens: Vec<SelectorToken>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &SelectorToken {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    fn advance(&mut self) -> &SelectorToken {
        let token = &self.tokens[self.pos.min(self.tokens.len() - 1)];
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        token
    }

    fn at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    /// Parse a selector list (comma-separated).
    pub fn parse_selector_list(&mut self) -> Result<SelectorList, SelectorError> {
        let mut selectors = Vec::new();

        loop {
            let selector = self.parse_selector()?;
            if !selector.is_empty() {
                selectors.push(selector);
            }

            if self.peek().token_type == TokenType::Comma {
                self.advance();
            } else {
                break;
            }
        }

        if selectors.is_empty() {
            return Err(SelectorError("Empty selector".to_string()));
        }

        Ok(SelectorList { selectors })
    }

    /// Parse a selector list until closing parenthesis (for :is() and :where()).
    fn parse_selector_list_until_close(&mut self) -> Result<Vec<Selector>, SelectorError> {
        let mut selectors = Vec::new();

        loop {
            // Skip whitespace
            while self.peek().token_type == TokenType::Combinator
                && self.peek().value.as_deref() == Some(" ")
            {
                self.advance();
            }

            // Check for end of list
            if self.at_end() || self.peek().token_type == TokenType::ParenClose {
                break;
            }

            let selector = self.parse_selector_until_comma_or_close()?;
            if !selector.is_empty() {
                selectors.push(selector);
            }

            if self.peek().token_type == TokenType::Comma {
                self.advance();
            } else {
                break;
            }
        }

        Ok(selectors)
    }

    /// Parse a single selector, stopping at comma or closing parenthesis.
    fn parse_selector_until_comma_or_close(&mut self) -> Result<Selector, SelectorError> {
        let mut parts = Vec::new();

        loop {
            // Check for combinator
            let combinator = if self.peek().token_type == TokenType::Combinator {
                let token = self.advance();
                let comb = match token.value.as_deref() {
                    Some(" ") => Combinator::Descendant,
                    Some(">") => Combinator::Child,
                    Some("+") => Combinator::Adjacent,
                    Some("~") => Combinator::Sibling,
                    _ => Combinator::Descendant,
                };
                Some(comb)
            } else if !parts.is_empty() {
                None
            } else {
                None
            };

            // Parse compound selector
            let simple_selectors = self.parse_compound_selector()?;
            if simple_selectors.is_empty() {
                break;
            }

            parts.push(SelectorPart {
                combinator: if parts.is_empty() { None } else { combinator },
                selectors: simple_selectors,
            });

            // Check if we should continue - stop at comma or close paren
            if self.at_end()
                || self.peek().token_type == TokenType::Comma
                || self.peek().token_type == TokenType::ParenClose
            {
                break;
            }
        }

        Ok(Selector { parts })
    }

    /// Parse a single selector (sequence of compound selectors).
    pub fn parse_selector(&mut self) -> Result<Selector, SelectorError> {
        let mut parts = Vec::new();

        loop {
            // Check for combinator
            let combinator = if self.peek().token_type == TokenType::Combinator {
                let token = self.advance();
                let comb = match token.value.as_deref() {
                    Some(" ") => Combinator::Descendant,
                    Some(">") => Combinator::Child,
                    Some("+") => Combinator::Adjacent,
                    Some("~") => Combinator::Sibling,
                    _ => Combinator::Descendant,
                };
                Some(comb)
            } else if !parts.is_empty() {
                None
            } else {
                None
            };

            // Parse compound selector
            let simple_selectors = self.parse_compound_selector()?;
            if simple_selectors.is_empty() {
                break;
            }

            parts.push(SelectorPart {
                combinator: if parts.is_empty() { None } else { combinator },
                selectors: simple_selectors,
            });

            // Check if we should continue
            if self.at_end() || self.peek().token_type == TokenType::Comma {
                break;
            }
        }

        Ok(Selector { parts })
    }

    /// Parse a compound selector (sequence of simple selectors).
    fn parse_compound_selector(&mut self) -> Result<Vec<SimpleSelector>, SelectorError> {
        let mut selectors = Vec::new();

        loop {
            let selector = match self.peek().token_type {
                TokenType::Universal => {
                    self.advance();
                    Some(SimpleSelector::Universal)
                }
                TokenType::Tag => {
                    let token = self.advance();
                    let name = token.value.clone().unwrap_or_default();
                    Some(SimpleSelector::Tag(name.to_ascii_lowercase()))
                }
                TokenType::Id => {
                    let token = self.advance();
                    let name = token.value.clone().unwrap_or_default();
                    Some(SimpleSelector::Id(name))
                }
                TokenType::Class => {
                    let token = self.advance();
                    let name = token.value.clone().unwrap_or_default();
                    Some(SimpleSelector::Class(name))
                }
                TokenType::AttrStart => {
                    Some(self.parse_attribute_selector()?)
                }
                TokenType::Colon => {
                    Some(self.parse_pseudo_selector()?)
                }
                _ => None,
            };

            match selector {
                Some(s) => selectors.push(s),
                None => break,
            }
        }

        Ok(selectors)
    }

    /// Parse an attribute selector.
    fn parse_attribute_selector(&mut self) -> Result<SimpleSelector, SelectorError> {
        self.advance(); // Skip [

        // Get attribute name
        let name = if self.peek().token_type == TokenType::Tag {
            let token = self.advance();
            token.value.clone().unwrap_or_default()
        } else {
            return Err(SelectorError("Expected attribute name".to_string()));
        };

        // Check for operator
        if self.peek().token_type == TokenType::AttrEnd {
            self.advance();
            return Ok(SimpleSelector::Attribute {
                name,
                op: AttributeOp::Exists,
                value: None,
                case_insensitive: false,
            });
        }

        let op = if self.peek().token_type == TokenType::AttrOp {
            let token = self.advance();
            match token.value.as_deref() {
                Some("=") => AttributeOp::Exact,
                Some("~=") => AttributeOp::Contains,
                Some("|=") => AttributeOp::DashPrefix,
                Some("^=") => AttributeOp::StartsWith,
                Some("$=") => AttributeOp::EndsWith,
                Some("*=") => AttributeOp::Substring,
                _ => return Err(SelectorError("Invalid attribute operator".to_string())),
            }
        } else {
            return Err(SelectorError("Expected attribute operator or ]".to_string()));
        };

        // Get value
        let value = if matches!(
            self.peek().token_type,
            TokenType::String | TokenType::Tag
        ) {
            let token = self.advance();
            token.value.clone()
        } else {
            return Err(SelectorError("Expected attribute value".to_string()));
        };

        // Skip any whitespace/combinator tokens before the case-insensitive flag
        while self.peek().token_type == TokenType::Combinator {
            self.advance();
        }

        // Check for case-insensitive flag (i or I)
        let case_insensitive = if self.peek().token_type == TokenType::Tag {
            let peeked = self.peek().value.as_deref();
            if peeked == Some("i") || peeked == Some("I") {
                self.advance();
                true
            } else {
                false
            }
        } else {
            false
        };

        // Expect ]
        if self.peek().token_type != TokenType::AttrEnd {
            return Err(SelectorError("Expected ]".to_string()));
        }
        self.advance();

        Ok(SimpleSelector::Attribute { name, op, value, case_insensitive })
    }

    /// Parse a pseudo-class selector.
    fn parse_pseudo_selector(&mut self) -> Result<SimpleSelector, SelectorError> {
        self.advance(); // Skip :

        // Get pseudo-class name
        let name = if self.peek().token_type == TokenType::Tag {
            let token = self.advance();
            token.value.clone().unwrap_or_default().to_ascii_lowercase()
        } else {
            return Err(SelectorError("Expected pseudo-class name".to_string()));
        };

        // Check for argument
        if self.peek().token_type == TokenType::ParenOpen {
            self.advance();

            if name == "not" {
                // Parse nested selector for :not()
                let inner = self.parse_selector()?;
                if self.peek().token_type != TokenType::ParenClose {
                    return Err(SelectorError("Expected ) in :not()".to_string()));
                }
                self.advance();
                return Ok(SimpleSelector::Not(Box::new(inner)));
            }

            if name == "is" || name == "where" {
                // Parse selector list for :is() and :where()
                let selectors = self.parse_selector_list_until_close()?;
                if self.peek().token_type != TokenType::ParenClose {
                    return Err(SelectorError(format!("Expected ) in :{}()", name)));
                }
                self.advance();
                return Ok(if name == "is" {
                    SimpleSelector::Is(selectors)
                } else {
                    SimpleSelector::Where(selectors)
                });
            }

            // Parse argument for other pseudo-classes
            let mut arg = String::new();
            let mut depth = 1;
            while depth > 0 && !self.at_end() {
                let token = self.advance();
                match token.token_type {
                    TokenType::ParenOpen => {
                        depth += 1;
                        arg.push('(');
                    }
                    TokenType::ParenClose => {
                        depth -= 1;
                        if depth > 0 {
                            arg.push(')');
                        }
                    }
                    _ => {
                        if let Some(ref v) = token.value {
                            arg.push_str(v);
                        }
                    }
                }
            }

            Ok(SimpleSelector::PseudoClass {
                name,
                argument: Some(arg.trim().to_string()),
            })
        } else {
            Ok(SimpleSelector::PseudoClass {
                name,
                argument: None,
            })
        }
    }
}

/// Parse a CSS selector string.
pub fn parse_selector(selector: &str) -> Result<SelectorList, SelectorError> {
    let mut tokenizer = SelectorTokenizer::new(selector);
    let tokens = tokenizer.tokenize()?;
    let mut parser = SelectorParser::new(tokens);
    parser.parse_selector_list()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tag() {
        let result = parse_selector("div").unwrap();
        assert_eq!(result.selectors.len(), 1);
        assert_eq!(result.selectors[0].parts.len(), 1);
        assert!(matches!(
            &result.selectors[0].parts[0].selectors[0],
            SimpleSelector::Tag(t) if t == "div"
        ));
    }

    #[test]
    fn test_parse_id() {
        let result = parse_selector("#main").unwrap();
        assert!(matches!(
            &result.selectors[0].parts[0].selectors[0],
            SimpleSelector::Id(id) if id == "main"
        ));
    }

    #[test]
    fn test_parse_class() {
        let result = parse_selector(".container").unwrap();
        assert!(matches!(
            &result.selectors[0].parts[0].selectors[0],
            SimpleSelector::Class(c) if c == "container"
        ));
    }

    #[test]
    fn test_parse_compound() {
        let result = parse_selector("div.active#main").unwrap();
        let selectors = &result.selectors[0].parts[0].selectors;
        assert_eq!(selectors.len(), 3);
        assert!(matches!(&selectors[0], SimpleSelector::Tag(t) if t == "div"));
        assert!(matches!(&selectors[1], SimpleSelector::Class(c) if c == "active"));
        assert!(matches!(&selectors[2], SimpleSelector::Id(id) if id == "main"));
    }

    #[test]
    fn test_parse_descendant() {
        let result = parse_selector("div span").unwrap();
        let parts = &result.selectors[0].parts;
        assert_eq!(parts.len(), 2);
        assert!(parts[0].combinator.is_none());
        assert_eq!(parts[1].combinator, Some(Combinator::Descendant));
    }

    #[test]
    fn test_parse_child() {
        let result = parse_selector("ul > li").unwrap();
        let parts = &result.selectors[0].parts;
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[1].combinator, Some(Combinator::Child));
    }

    #[test]
    fn test_parse_attribute_exists() {
        let result = parse_selector("[disabled]").unwrap();
        assert!(matches!(
            &result.selectors[0].parts[0].selectors[0],
            SimpleSelector::Attribute { name, op: AttributeOp::Exists, value: None, .. } if name == "disabled"
        ));
    }

    #[test]
    fn test_parse_attribute_equals() {
        let result = parse_selector("[type=\"text\"]").unwrap();
        assert!(matches!(
            &result.selectors[0].parts[0].selectors[0],
            SimpleSelector::Attribute { name, op: AttributeOp::Exact, value: Some(v), .. }
            if name == "type" && v == "text"
        ));
    }

    #[test]
    fn test_parse_selector_list() {
        let result = parse_selector("div, span, p").unwrap();
        assert_eq!(result.selectors.len(), 3);
    }

    #[test]
    fn test_parse_pseudo_class() {
        let result = parse_selector(":first-child").unwrap();
        assert!(matches!(
            &result.selectors[0].parts[0].selectors[0],
            SimpleSelector::PseudoClass { name, argument: None } if name == "first-child"
        ));
    }
}

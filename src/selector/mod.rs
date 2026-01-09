//! CSS selector module.
//!
//! Provides CSS selector parsing and DOM querying functionality.

mod lexer;
mod matcher;
mod parser;

pub use lexer::{SelectorToken, SelectorTokenizer, TokenType};
pub use matcher::{matches_selector, query, query_all};
pub use parser::{parse_selector, AttributeOp, Combinator, Selector, SelectorPart, SimpleSelector};

//! HTML5 tokenizer implementation.
//!
//! This module implements HTML5 tokenization using nom parser combinators.

mod entities;
mod input;
mod nom_parser;
mod state;
mod tokens;

pub use entities::decode_entity;
pub use input::InputStream;
pub use nom_parser::{tokenize, NomTokenizer};
pub use state::State;
pub use tokens::{
    CharacterTokens, CommentToken, Doctype, DoctypeToken, EOFToken, Tag, TagKind, Token,
    TokenSinkResult,
};

/// Re-export NomTokenizer as Tokenizer for API compatibility.
pub type Tokenizer<'a> = NomTokenizer<'a>;

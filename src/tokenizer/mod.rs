//! HTML5 tokenizer implementation.
//!
//! This module implements the WHATWG HTML5 tokenization algorithm.

mod entities;
mod input;
mod state;
mod tokens;

pub use entities::decode_entity;
pub use input::InputStream;
pub use state::{State, Tokenizer};
pub use tokens::{
    CharacterTokens, CommentToken, Doctype, DoctypeToken, EOFToken, Tag, TagKind, Token,
    TokenSinkResult,
};

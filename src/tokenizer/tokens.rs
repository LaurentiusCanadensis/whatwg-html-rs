//! Token types emitted by the HTML5 tokenizer.

use compact_str::CompactString;
use smallvec::SmallVec;

/// The kind of a tag token (start or end).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagKind {
    Start,
    End,
}

/// A start or end tag token.
#[derive(Debug, Clone)]
pub struct Tag {
    /// Whether this is a start or end tag.
    pub kind: TagKind,
    /// The tag name (lowercase for HTML elements).
    pub name: CompactString,
    /// Attributes as (name, value) pairs. Value is None for boolean attributes.
    pub attrs: SmallVec<[(CompactString, Option<CompactString>); 4]>,
    /// Whether the tag is self-closing (e.g., `<br/>`).
    pub self_closing: bool,
    /// Start position in the source (byte offset), if tracked.
    pub start_pos: Option<u32>,
}

impl Tag {
    /// Create a new start tag with the given name.
    pub fn start(name: impl Into<CompactString>) -> Self {
        Self {
            kind: TagKind::Start,
            name: name.into(),
            attrs: SmallVec::new(),
            self_closing: false,
            start_pos: None,
        }
    }

    /// Create a new end tag with the given name.
    pub fn end(name: impl Into<CompactString>) -> Self {
        Self {
            kind: TagKind::End,
            name: name.into(),
            attrs: SmallVec::new(),
            self_closing: false,
            start_pos: None,
        }
    }

    /// Check if this is a start tag.
    #[inline]
    pub fn is_start(&self) -> bool {
        self.kind == TagKind::Start
    }

    /// Check if this is an end tag.
    #[inline]
    pub fn is_end(&self) -> bool {
        self.kind == TagKind::End
    }

    /// Add an attribute to the tag.
    pub fn add_attr(&mut self, name: impl Into<CompactString>, value: Option<CompactString>) {
        self.attrs.push((name.into(), value));
    }

    /// Get an attribute value by name.
    pub fn get_attr(&self, name: &str) -> Option<Option<&str>> {
        self.attrs
            .iter()
            .find(|(n, _)| n.as_str() == name)
            .map(|(_, v)| v.as_deref())
    }

    /// Check if the tag has a specific attribute.
    pub fn has_attr(&self, name: &str) -> bool {
        self.attrs.iter().any(|(n, _)| n.as_str() == name)
    }
}

/// Character data token (text content).
#[derive(Debug, Clone)]
pub struct CharacterTokens {
    /// The text data.
    pub data: CompactString,
}

impl CharacterTokens {
    /// Create a new character token.
    pub fn new(data: impl Into<CompactString>) -> Self {
        Self { data: data.into() }
    }

    /// Append text to this token.
    pub fn append(&mut self, s: &str) {
        self.data.push_str(s);
    }

    /// Append a character to this token.
    pub fn push(&mut self, c: char) {
        self.data.push(c);
    }
}

/// A comment token.
#[derive(Debug, Clone)]
pub struct CommentToken {
    /// The comment data (content between `<!--` and `-->`).
    pub data: CompactString,
    /// Start position in the source (byte offset), if tracked.
    pub start_pos: Option<u32>,
}

impl CommentToken {
    /// Create a new comment token.
    pub fn new(data: impl Into<CompactString>) -> Self {
        Self {
            data: data.into(),
            start_pos: None,
        }
    }
}

/// DOCTYPE information.
#[derive(Debug, Clone, Default)]
pub struct Doctype {
    /// The DOCTYPE name (usually "html").
    pub name: Option<CompactString>,
    /// The public identifier.
    pub public_id: Option<CompactString>,
    /// The system identifier.
    pub system_id: Option<CompactString>,
    /// Whether to force quirks mode.
    pub force_quirks: bool,
}

impl Doctype {
    /// Create a new DOCTYPE with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a standard HTML5 DOCTYPE.
    pub fn html5() -> Self {
        Self {
            name: Some("html".into()),
            public_id: None,
            system_id: None,
            force_quirks: false,
        }
    }
}

/// A DOCTYPE token.
#[derive(Debug, Clone)]
pub struct DoctypeToken {
    /// The DOCTYPE data.
    pub doctype: Doctype,
}

impl DoctypeToken {
    /// Create a new DOCTYPE token.
    pub fn new(doctype: Doctype) -> Self {
        Self { doctype }
    }
}

/// End of file token.
#[derive(Debug, Clone, Copy)]
pub struct EOFToken;

/// Result of processing a token by a tree builder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenSinkResult {
    /// Continue normal parsing.
    Continue,
    /// Switch to plaintext mode.
    Plaintext,
    /// Switch to raw text mode (script/style).
    RawText,
    /// Switch to RCDATA mode (textarea/title).
    RCData,
}

/// A token emitted by the tokenizer.
#[derive(Debug, Clone)]
pub enum Token {
    /// A start or end tag.
    Tag(Tag),
    /// Character data (text).
    Characters(CharacterTokens),
    /// A comment.
    Comment(CommentToken),
    /// A DOCTYPE declaration.
    Doctype(DoctypeToken),
    /// End of file.
    EOF,
}

impl Token {
    /// Check if this is a start tag with the given name.
    pub fn is_start_tag(&self, name: &str) -> bool {
        matches!(self, Token::Tag(t) if t.is_start() && t.name == name)
    }

    /// Check if this is an end tag with the given name.
    pub fn is_end_tag(&self, name: &str) -> bool {
        matches!(self, Token::Tag(t) if t.is_end() && t.name == name)
    }

    /// Check if this is a character token.
    pub fn is_characters(&self) -> bool {
        matches!(self, Token::Characters(_))
    }

    /// Check if this is an EOF token.
    pub fn is_eof(&self) -> bool {
        matches!(self, Token::EOF)
    }

    /// Get the tag if this is a tag token.
    pub fn as_tag(&self) -> Option<&Tag> {
        match self {
            Token::Tag(t) => Some(t),
            _ => None,
        }
    }

    /// Get the tag mutably if this is a tag token.
    pub fn as_tag_mut(&mut self) -> Option<&mut Tag> {
        match self {
            Token::Tag(t) => Some(t),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_creation() {
        let mut tag = Tag::start("div");
        tag.add_attr("class", Some("container".into()));
        tag.add_attr("id", Some("main".into()));
        tag.add_attr("disabled", None);

        assert!(tag.is_start());
        assert!(!tag.is_end());
        assert_eq!(tag.name.as_str(), "div");
        assert_eq!(tag.get_attr("class"), Some(Some("container")));
        assert_eq!(tag.get_attr("disabled"), Some(None));
        assert!(tag.has_attr("id"));
        assert!(!tag.has_attr("missing"));
    }

    #[test]
    fn test_character_tokens() {
        let mut chars = CharacterTokens::new("Hello");
        chars.append(", ");
        chars.push('W');
        chars.append("orld");

        assert_eq!(chars.data.as_str(), "Hello, World");
    }

    #[test]
    fn test_doctype() {
        let doctype = Doctype::html5();
        assert_eq!(doctype.name.as_deref(), Some("html"));
        assert!(doctype.public_id.is_none());
        assert!(doctype.system_id.is_none());
        assert!(!doctype.force_quirks);
    }

    #[test]
    fn test_token_matching() {
        let token = Token::Tag(Tag::start("script"));
        assert!(token.is_start_tag("script"));
        assert!(!token.is_start_tag("style"));
        assert!(!token.is_end_tag("script"));

        let token = Token::Tag(Tag::end("div"));
        assert!(token.is_end_tag("div"));
        assert!(!token.is_start_tag("div"));
    }
}

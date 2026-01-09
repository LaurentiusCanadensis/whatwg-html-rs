//! Tokenizer state machine.
//!
//! Implements the HTML5 tokenization algorithm per WHATWG spec.

use compact_str::CompactString;
use smallvec::SmallVec;

// Entity decoding will be used in full implementation
#[allow(unused_imports)]
use super::entities::{decode_entity, decode_numeric, is_legacy_entity};
use super::input::InputStream;
use super::tokens::{CharacterTokens, CommentToken, Doctype, DoctypeToken, Tag, TagKind, Token};
use crate::error::ParseError;

/// Tokenizer state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum State {
    Data = 0,
    TagOpen = 1,
    EndTagOpen = 2,
    TagName = 3,
    BeforeAttributeName = 4,
    AttributeName = 5,
    AfterAttributeName = 6,
    BeforeAttributeValue = 7,
    AttributeValueDoubleQuoted = 8,
    AttributeValueSingleQuoted = 9,
    AttributeValueUnquoted = 10,
    AfterAttributeValueQuoted = 11,
    SelfClosingStartTag = 12,
    MarkupDeclarationOpen = 13,
    CommentStart = 14,
    CommentStartDash = 15,
    Comment = 16,
    CommentEndDash = 17,
    CommentEnd = 18,
    CommentEndBang = 19,
    BogusComment = 20,
    Doctype = 21,
    BeforeDoctypeName = 22,
    DoctypeName = 23,
    AfterDoctypeName = 24,
    BogusDoctype = 25,
    AfterDoctypePublicKeyword = 26,
    AfterDoctypeSystemKeyword = 27,
    BeforeDoctypePublicIdentifier = 28,
    DoctypePublicIdentifierDoubleQuoted = 29,
    DoctypePublicIdentifierSingleQuoted = 30,
    AfterDoctypePublicIdentifier = 31,
    BetweenDoctypePublicAndSystemIdentifiers = 32,
    BeforeDoctypeSystemIdentifier = 33,
    DoctypeSystemIdentifierDoubleQuoted = 34,
    DoctypeSystemIdentifierSingleQuoted = 35,
    AfterDoctypeSystemIdentifier = 36,
    CdataSection = 37,
    CdataSectionBracket = 38,
    CdataSectionEnd = 39,
    RcData = 40,
    RcDataLessThanSign = 41,
    RcDataEndTagOpen = 42,
    RcDataEndTagName = 43,
    RawText = 44,
    RawTextLessThanSign = 45,
    RawTextEndTagOpen = 46,
    RawTextEndTagName = 47,
    PlainText = 48,
    ScriptDataEscaped = 49,
    ScriptDataEscapedDash = 50,
    ScriptDataEscapedDashDash = 51,
    ScriptDataEscapedLessThanSign = 52,
    ScriptDataEscapedEndTagOpen = 53,
    ScriptDataEscapedEndTagName = 54,
    ScriptDataDoubleEscapeStart = 55,
    ScriptDataDoubleEscaped = 56,
    ScriptDataDoubleEscapedDash = 57,
    ScriptDataDoubleEscapedDashDash = 58,
    ScriptDataDoubleEscapedLessThanSign = 59,
    ScriptDataDoubleEscapeEnd = 60,
}

/// Configuration options for the tokenizer.
#[derive(Debug, Clone, Default)]
pub struct TokenizerOpts {
    /// Discard BOM at start of input.
    pub discard_bom: bool,
    /// Report all errors with exact positions.
    pub exact_errors: bool,
    /// Initial raw text tag name (for fragment parsing).
    pub initial_rawtext_tag: Option<String>,
    /// Initial state (for fragment parsing).
    pub initial_state: Option<State>,
}

/// HTML5 tokenizer.
pub struct Tokenizer<'a> {
    /// Input stream.
    input: InputStream<'a>,
    /// Current state.
    state: State,
    /// Whether to reconsume the current character.
    reconsume: bool,
    /// Current character being processed.
    current_char: Option<char>,

    /// Collected parse errors.
    pub errors: Vec<ParseError>,
    /// Whether to collect errors.
    pub collect_errors: bool,

    // Token building state
    /// Text buffer for character tokens.
    text_buffer: String,
    /// Current tag being built.
    current_tag: Option<Tag>,
    /// Current comment being built.
    current_comment: String,
    /// Current DOCTYPE being built.
    current_doctype: Doctype,
    /// Current attribute name.
    current_attr_name: String,
    /// Current attribute value.
    current_attr_value: String,
    /// Whether current attr value has ampersand (needs entity decoding).
    current_attr_value_has_amp: bool,
    /// Temporary buffer for special states.
    temp_buffer: String,
    /// Last emitted start tag name (for appropriate end tag matching).
    last_start_tag_name: Option<CompactString>,
    /// Token start position.
    token_start_pos: u32,

    /// Pending tokens to emit.
    pending_tokens: Vec<Token>,
}

impl<'a> Tokenizer<'a> {
    /// Create a new tokenizer for the given input.
    pub fn new(input: &'a str) -> Self {
        let mut stream = InputStream::new(input);

        // Discard BOM if present
        if stream.peek() == Some('\u{FEFF}') {
            stream.next();
        }

        Self {
            input: stream,
            state: State::Data,
            reconsume: false,
            current_char: None,
            errors: Vec::new(),
            collect_errors: false,
            text_buffer: String::new(),
            current_tag: None,
            current_comment: String::new(),
            current_doctype: Doctype::default(),
            current_attr_name: String::new(),
            current_attr_value: String::new(),
            current_attr_value_has_amp: false,
            temp_buffer: String::new(),
            last_start_tag_name: None,
            token_start_pos: 0,
            pending_tokens: Vec::new(),
        }
    }

    /// Create a tokenizer with options.
    pub fn with_opts(input: &'a str, opts: TokenizerOpts) -> Self {
        let mut tokenizer = Self::new(input);
        if let Some(state) = opts.initial_state {
            tokenizer.state = state;
        }
        tokenizer
    }

    /// Get the next token.
    pub fn next_token(&mut self) -> Option<Token> {
        // Return pending tokens first
        if let Some(token) = self.pending_tokens.pop() {
            return Some(token);
        }

        loop {
            if self.input.is_empty() && !self.reconsume {
                // Emit any remaining text
                if !self.text_buffer.is_empty() {
                    let text = std::mem::take(&mut self.text_buffer);
                    return Some(Token::Characters(CharacterTokens::new(text)));
                }
                return Some(Token::EOF);
            }

            // Get next character
            if !self.reconsume {
                self.current_char = self.input.next();
            }
            self.reconsume = false;

            // Process current state
            if let Some(token) = self.process_state() {
                return Some(token);
            }

            // Check for pending tokens
            if let Some(token) = self.pending_tokens.pop() {
                return Some(token);
            }
        }
    }

    /// Process the current state and optionally emit a token.
    fn process_state(&mut self) -> Option<Token> {
        match self.state {
            State::Data => self.data_state(),
            State::TagOpen => self.tag_open_state(),
            State::EndTagOpen => self.end_tag_open_state(),
            State::TagName => self.tag_name_state(),
            State::BeforeAttributeName => self.before_attribute_name_state(),
            State::AttributeName => self.attribute_name_state(),
            State::AfterAttributeName => self.after_attribute_name_state(),
            State::BeforeAttributeValue => self.before_attribute_value_state(),
            State::AttributeValueDoubleQuoted => self.attribute_value_double_quoted_state(),
            State::AttributeValueSingleQuoted => self.attribute_value_single_quoted_state(),
            State::AttributeValueUnquoted => self.attribute_value_unquoted_state(),
            State::AfterAttributeValueQuoted => self.after_attribute_value_quoted_state(),
            State::SelfClosingStartTag => self.self_closing_start_tag_state(),
            State::MarkupDeclarationOpen => self.markup_declaration_open_state(),
            State::CommentStart => self.comment_start_state(),
            State::CommentStartDash => self.comment_start_dash_state(),
            State::Comment => self.comment_state(),
            State::CommentEndDash => self.comment_end_dash_state(),
            State::CommentEnd => self.comment_end_state(),
            State::CommentEndBang => self.comment_end_bang_state(),
            State::BogusComment => self.bogus_comment_state(),
            State::Doctype => self.doctype_state(),
            State::BeforeDoctypeName => self.before_doctype_name_state(),
            State::DoctypeName => self.doctype_name_state(),
            State::AfterDoctypeName => self.after_doctype_name_state(),
            State::BogusDoctype => self.bogus_doctype_state(),
            State::AfterDoctypePublicKeyword => self.after_doctype_public_keyword_state(),
            State::AfterDoctypeSystemKeyword => self.after_doctype_system_keyword_state(),
            State::BeforeDoctypePublicIdentifier => self.before_doctype_public_identifier_state(),
            State::DoctypePublicIdentifierDoubleQuoted => {
                self.doctype_public_identifier_double_quoted_state()
            }
            State::DoctypePublicIdentifierSingleQuoted => {
                self.doctype_public_identifier_single_quoted_state()
            }
            State::AfterDoctypePublicIdentifier => self.after_doctype_public_identifier_state(),
            State::BetweenDoctypePublicAndSystemIdentifiers => {
                self.between_doctype_public_and_system_identifiers_state()
            }
            State::BeforeDoctypeSystemIdentifier => self.before_doctype_system_identifier_state(),
            State::DoctypeSystemIdentifierDoubleQuoted => {
                self.doctype_system_identifier_double_quoted_state()
            }
            State::DoctypeSystemIdentifierSingleQuoted => {
                self.doctype_system_identifier_single_quoted_state()
            }
            State::AfterDoctypeSystemIdentifier => self.after_doctype_system_identifier_state(),
            State::CdataSection => self.cdata_section_state(),
            State::CdataSectionBracket => self.cdata_section_bracket_state(),
            State::CdataSectionEnd => self.cdata_section_end_state(),
            State::RcData => self.rcdata_state(),
            State::RcDataLessThanSign => self.rcdata_less_than_sign_state(),
            State::RcDataEndTagOpen => self.rcdata_end_tag_open_state(),
            State::RcDataEndTagName => self.rcdata_end_tag_name_state(),
            State::RawText => self.rawtext_state(),
            State::RawTextLessThanSign => self.rawtext_less_than_sign_state(),
            State::RawTextEndTagOpen => self.rawtext_end_tag_open_state(),
            State::RawTextEndTagName => self.rawtext_end_tag_name_state(),
            State::PlainText => self.plaintext_state(),
            _ => None, // Script data states to be implemented
        }
    }

    /// Emit a parse error.
    fn emit_error(&mut self, code: &str) {
        if self.collect_errors {
            let (line, col) = self.input.position();
            self.errors.push(ParseError::new(code, Some(line), Some(col)));
        }
    }

    /// Flush text buffer and return character token if non-empty.
    fn flush_text(&mut self) -> Option<Token> {
        if !self.text_buffer.is_empty() {
            let text = std::mem::take(&mut self.text_buffer);
            Some(Token::Characters(CharacterTokens::new(text)))
        } else {
            None
        }
    }

    /// Start building a new tag.
    fn start_tag(&mut self, kind: TagKind) {
        self.current_tag = Some(Tag {
            kind,
            name: CompactString::new(""),
            attrs: SmallVec::new(),
            self_closing: false,
            start_pos: Some(self.input.offset() as u32),
        });
    }

    /// Emit the current tag token.
    fn emit_tag(&mut self) -> Option<Token> {
        if let Some(mut tag) = self.current_tag.take() {
            // Track last start tag name for appropriate end tag matching
            if tag.kind == TagKind::Start {
                self.last_start_tag_name = Some(tag.name.clone());
            }
            Some(Token::Tag(tag))
        } else {
            None
        }
    }

    /// Emit a comment token.
    fn emit_comment(&mut self) -> Option<Token> {
        let data = std::mem::take(&mut self.current_comment);
        Some(Token::Comment(CommentToken::new(data)))
    }

    /// Emit a DOCTYPE token.
    fn emit_doctype(&mut self) -> Option<Token> {
        let doctype = std::mem::take(&mut self.current_doctype);
        Some(Token::Doctype(DoctypeToken::new(doctype)))
    }

    /// Check if the temp buffer matches the last start tag name (for appropriate end tag).
    fn is_appropriate_end_tag(&self) -> bool {
        if let Some(ref last) = self.last_start_tag_name {
            self.temp_buffer.eq_ignore_ascii_case(last)
        } else {
            false
        }
    }

    // ========================================================================
    // State handlers
    // ========================================================================

    fn data_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('&') => {
                // Entity reference - simplified handling
                self.text_buffer.push('&');
                None
            }
            Some('<') => {
                let token = self.flush_text();
                self.state = State::TagOpen;
                token
            }
            Some('\0') => {
                self.emit_error("unexpected-null-character");
                self.text_buffer.push('\u{FFFD}');
                None
            }
            Some(c) => {
                self.text_buffer.push(c);
                None
            }
            None => self.flush_text(),
        }
    }

    fn tag_open_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('!') => {
                self.state = State::MarkupDeclarationOpen;
                None
            }
            Some('/') => {
                self.state = State::EndTagOpen;
                None
            }
            Some(c) if c.is_ascii_alphabetic() => {
                self.start_tag(TagKind::Start);
                self.reconsume = true;
                self.state = State::TagName;
                None
            }
            Some('?') => {
                self.emit_error("unexpected-question-mark-instead-of-tag-name");
                self.current_comment.clear();
                self.reconsume = true;
                self.state = State::BogusComment;
                None
            }
            None => {
                self.emit_error("eof-before-tag-name");
                self.text_buffer.push('<');
                self.flush_text()
            }
            Some(_) => {
                self.emit_error("invalid-first-character-of-tag-name");
                self.text_buffer.push('<');
                self.reconsume = true;
                self.state = State::Data;
                None
            }
        }
    }

    fn end_tag_open_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some(c) if c.is_ascii_alphabetic() => {
                self.start_tag(TagKind::End);
                self.reconsume = true;
                self.state = State::TagName;
                None
            }
            Some('>') => {
                self.emit_error("missing-end-tag-name");
                self.state = State::Data;
                None
            }
            None => {
                self.emit_error("eof-before-tag-name");
                self.text_buffer.push_str("</");
                self.flush_text()
            }
            Some(_) => {
                self.emit_error("invalid-first-character-of-tag-name");
                self.current_comment.clear();
                self.reconsume = true;
                self.state = State::BogusComment;
                None
            }
        }
    }

    fn tag_name_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                self.state = State::BeforeAttributeName;
                None
            }
            Some('/') => {
                self.state = State::SelfClosingStartTag;
                None
            }
            Some('>') => {
                self.state = State::Data;
                self.emit_tag()
            }
            Some('\0') => {
                self.emit_error("unexpected-null-character");
                if let Some(ref mut tag) = self.current_tag {
                    tag.name.push('\u{FFFD}');
                }
                None
            }
            Some(c) => {
                if let Some(ref mut tag) = self.current_tag {
                    tag.name.push(c.to_ascii_lowercase());
                }
                None
            }
            None => {
                self.emit_error("eof-in-tag");
                None
            }
        }
    }

    fn before_attribute_name_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => None, // Ignore
            Some('/') | Some('>') | None => {
                self.reconsume = true;
                self.state = State::AfterAttributeName;
                None
            }
            Some('=') => {
                self.emit_error("unexpected-equals-sign-before-attribute-name");
                self.current_attr_name.clear();
                self.current_attr_name.push('=');
                self.current_attr_value.clear();
                self.state = State::AttributeName;
                None
            }
            Some(_) => {
                self.current_attr_name.clear();
                self.current_attr_value.clear();
                self.reconsume = true;
                self.state = State::AttributeName;
                None
            }
        }
    }

    fn attribute_name_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') | Some('/') | Some('>') | None => {
                self.reconsume = true;
                self.state = State::AfterAttributeName;
                None
            }
            Some('=') => {
                self.state = State::BeforeAttributeValue;
                None
            }
            Some('\0') => {
                self.emit_error("unexpected-null-character");
                self.current_attr_name.push('\u{FFFD}');
                None
            }
            Some('"') | Some('\'') | Some('<') => {
                self.emit_error("unexpected-character-in-attribute-name");
                self.current_attr_name
                    .push(self.current_char.unwrap().to_ascii_lowercase());
                None
            }
            Some(c) => {
                self.current_attr_name.push(c.to_ascii_lowercase());
                None
            }
        }
    }

    fn after_attribute_name_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => None, // Ignore
            Some('/') => {
                // Add attribute without value
                self.add_current_attribute();
                self.state = State::SelfClosingStartTag;
                None
            }
            Some('=') => {
                self.state = State::BeforeAttributeValue;
                None
            }
            Some('>') => {
                self.add_current_attribute();
                self.state = State::Data;
                self.emit_tag()
            }
            None => {
                self.emit_error("eof-in-tag");
                None
            }
            Some(_) => {
                self.add_current_attribute();
                self.current_attr_name.clear();
                self.current_attr_value.clear();
                self.reconsume = true;
                self.state = State::AttributeName;
                None
            }
        }
    }

    fn before_attribute_value_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => None, // Ignore
            Some('"') => {
                self.state = State::AttributeValueDoubleQuoted;
                None
            }
            Some('\'') => {
                self.state = State::AttributeValueSingleQuoted;
                None
            }
            Some('>') => {
                self.emit_error("missing-attribute-value");
                self.add_current_attribute();
                self.state = State::Data;
                self.emit_tag()
            }
            Some(_) => {
                self.reconsume = true;
                self.state = State::AttributeValueUnquoted;
                None
            }
            None => {
                self.emit_error("eof-in-tag");
                None
            }
        }
    }

    fn attribute_value_double_quoted_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('"') => {
                self.add_current_attribute();
                self.state = State::AfterAttributeValueQuoted;
                None
            }
            Some('&') => {
                self.current_attr_value_has_amp = true;
                self.current_attr_value.push('&');
                None
            }
            Some('\0') => {
                self.emit_error("unexpected-null-character");
                self.current_attr_value.push('\u{FFFD}');
                None
            }
            Some(c) => {
                self.current_attr_value.push(c);
                None
            }
            None => {
                self.emit_error("eof-in-tag");
                None
            }
        }
    }

    fn attribute_value_single_quoted_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('\'') => {
                self.add_current_attribute();
                self.state = State::AfterAttributeValueQuoted;
                None
            }
            Some('&') => {
                self.current_attr_value_has_amp = true;
                self.current_attr_value.push('&');
                None
            }
            Some('\0') => {
                self.emit_error("unexpected-null-character");
                self.current_attr_value.push('\u{FFFD}');
                None
            }
            Some(c) => {
                self.current_attr_value.push(c);
                None
            }
            None => {
                self.emit_error("eof-in-tag");
                None
            }
        }
    }

    fn attribute_value_unquoted_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                self.add_current_attribute();
                self.state = State::BeforeAttributeName;
                None
            }
            Some('&') => {
                self.current_attr_value_has_amp = true;
                self.current_attr_value.push('&');
                None
            }
            Some('>') => {
                self.add_current_attribute();
                self.state = State::Data;
                self.emit_tag()
            }
            Some('\0') => {
                self.emit_error("unexpected-null-character");
                self.current_attr_value.push('\u{FFFD}');
                None
            }
            Some('"') | Some('\'') | Some('<') | Some('=') | Some('`') => {
                self.emit_error("unexpected-character-in-unquoted-attribute-value");
                self.current_attr_value.push(self.current_char.unwrap());
                None
            }
            Some(c) => {
                self.current_attr_value.push(c);
                None
            }
            None => {
                self.emit_error("eof-in-tag");
                None
            }
        }
    }

    fn after_attribute_value_quoted_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                self.state = State::BeforeAttributeName;
                None
            }
            Some('/') => {
                self.state = State::SelfClosingStartTag;
                None
            }
            Some('>') => {
                self.state = State::Data;
                self.emit_tag()
            }
            None => {
                self.emit_error("eof-in-tag");
                None
            }
            Some(_) => {
                self.emit_error("missing-whitespace-between-attributes");
                self.reconsume = true;
                self.state = State::BeforeAttributeName;
                None
            }
        }
    }

    fn self_closing_start_tag_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('>') => {
                if let Some(ref mut tag) = self.current_tag {
                    tag.self_closing = true;
                }
                self.state = State::Data;
                self.emit_tag()
            }
            None => {
                self.emit_error("eof-in-tag");
                None
            }
            Some(_) => {
                self.emit_error("unexpected-solidus-in-tag");
                self.reconsume = true;
                self.state = State::BeforeAttributeName;
                None
            }
        }
    }

    fn markup_declaration_open_state(&mut self) -> Option<Token> {
        // Check for "--" (comment), "DOCTYPE", or "[CDATA["
        // Note: current_char is the first char after "<!"
        match self.current_char {
            Some('-') if self.input.peek() == Some('-') => {
                self.input.next(); // consume second '-'
                self.current_comment.clear();
                self.state = State::CommentStart;
            }
            Some('D') | Some('d') => {
                // Check for "OCTYPE" (we already have the D)
                if self.input.starts_with_ignore_case("OCTYPE") {
                    self.input.skip(6);
                    self.state = State::Doctype;
                } else {
                    self.emit_error("incorrectly-opened-comment");
                    self.current_comment.clear();
                    self.current_comment.push(self.current_char.unwrap());
                    self.state = State::BogusComment;
                }
            }
            Some('[') if self.input.starts_with("CDATA[") => {
                self.input.skip(6);
                // TODO: Check if in foreign content
                self.emit_error("cdata-in-html-content");
                self.current_comment.clear();
                self.state = State::BogusComment;
            }
            _ => {
                self.emit_error("incorrectly-opened-comment");
                self.current_comment.clear();
                if let Some(c) = self.current_char {
                    self.current_comment.push(c);
                }
                self.state = State::BogusComment;
            }
        }
        None
    }

    fn comment_start_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('-') => {
                self.state = State::CommentStartDash;
                None
            }
            Some('>') => {
                self.emit_error("abrupt-closing-of-empty-comment");
                self.state = State::Data;
                self.emit_comment()
            }
            _ => {
                self.reconsume = true;
                self.state = State::Comment;
                None
            }
        }
    }

    fn comment_start_dash_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('-') => {
                self.state = State::CommentEnd;
                None
            }
            Some('>') => {
                self.emit_error("abrupt-closing-of-empty-comment");
                self.state = State::Data;
                self.emit_comment()
            }
            None => {
                self.emit_error("eof-in-comment");
                self.emit_comment()
            }
            _ => {
                self.current_comment.push('-');
                self.reconsume = true;
                self.state = State::Comment;
                None
            }
        }
    }

    fn comment_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('<') => {
                self.current_comment.push('<');
                None
            }
            Some('-') => {
                self.state = State::CommentEndDash;
                None
            }
            Some('\0') => {
                self.emit_error("unexpected-null-character");
                self.current_comment.push('\u{FFFD}');
                None
            }
            Some(c) => {
                self.current_comment.push(c);
                None
            }
            None => {
                self.emit_error("eof-in-comment");
                self.emit_comment()
            }
        }
    }

    fn comment_end_dash_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('-') => {
                self.state = State::CommentEnd;
                None
            }
            None => {
                self.emit_error("eof-in-comment");
                self.emit_comment()
            }
            _ => {
                self.current_comment.push('-');
                self.reconsume = true;
                self.state = State::Comment;
                None
            }
        }
    }

    fn comment_end_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('>') => {
                self.state = State::Data;
                self.emit_comment()
            }
            Some('!') => {
                self.state = State::CommentEndBang;
                None
            }
            Some('-') => {
                self.current_comment.push('-');
                None
            }
            None => {
                self.emit_error("eof-in-comment");
                self.emit_comment()
            }
            _ => {
                self.current_comment.push_str("--");
                self.reconsume = true;
                self.state = State::Comment;
                None
            }
        }
    }

    fn comment_end_bang_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('-') => {
                self.current_comment.push_str("--!");
                self.state = State::CommentEndDash;
                None
            }
            Some('>') => {
                self.emit_error("incorrectly-closed-comment");
                self.state = State::Data;
                self.emit_comment()
            }
            None => {
                self.emit_error("eof-in-comment");
                self.emit_comment()
            }
            _ => {
                self.current_comment.push_str("--!");
                self.reconsume = true;
                self.state = State::Comment;
                None
            }
        }
    }

    fn bogus_comment_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('>') => {
                self.state = State::Data;
                self.emit_comment()
            }
            Some('\0') => {
                self.emit_error("unexpected-null-character");
                self.current_comment.push('\u{FFFD}');
                None
            }
            Some(c) => {
                self.current_comment.push(c);
                None
            }
            None => self.emit_comment(),
        }
    }

    fn doctype_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                self.state = State::BeforeDoctypeName;
                None
            }
            Some('>') => {
                self.reconsume = true;
                self.state = State::BeforeDoctypeName;
                None
            }
            None => {
                self.emit_error("eof-in-doctype");
                self.current_doctype = Doctype {
                    force_quirks: true,
                    ..Default::default()
                };
                self.emit_doctype()
            }
            _ => {
                self.emit_error("missing-whitespace-before-doctype-name");
                self.reconsume = true;
                self.state = State::BeforeDoctypeName;
                None
            }
        }
    }

    fn before_doctype_name_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => None, // Ignore
            Some('\0') => {
                self.emit_error("unexpected-null-character");
                self.current_doctype = Doctype::default();
                self.current_doctype.name = Some("\u{FFFD}".into());
                self.state = State::DoctypeName;
                None
            }
            Some('>') => {
                self.emit_error("missing-doctype-name");
                self.current_doctype = Doctype {
                    force_quirks: true,
                    ..Default::default()
                };
                self.state = State::Data;
                self.emit_doctype()
            }
            Some(c) => {
                self.current_doctype = Doctype::default();
                self.current_doctype.name = Some(c.to_ascii_lowercase().to_string().into());
                self.state = State::DoctypeName;
                None
            }
            None => {
                self.emit_error("eof-in-doctype");
                self.current_doctype = Doctype {
                    force_quirks: true,
                    ..Default::default()
                };
                self.emit_doctype()
            }
        }
    }

    fn doctype_name_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                self.state = State::AfterDoctypeName;
                None
            }
            Some('>') => {
                self.state = State::Data;
                self.emit_doctype()
            }
            Some('\0') => {
                self.emit_error("unexpected-null-character");
                if let Some(ref mut name) = self.current_doctype.name {
                    name.push('\u{FFFD}');
                }
                None
            }
            Some(c) => {
                if let Some(ref mut name) = self.current_doctype.name {
                    name.push(c.to_ascii_lowercase());
                }
                None
            }
            None => {
                self.emit_error("eof-in-doctype");
                self.current_doctype.force_quirks = true;
                self.emit_doctype()
            }
        }
    }

    fn after_doctype_name_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => None, // Ignore
            Some('>') => {
                self.state = State::Data;
                self.emit_doctype()
            }
            None => {
                self.emit_error("eof-in-doctype");
                self.current_doctype.force_quirks = true;
                self.emit_doctype()
            }
            Some(_) => {
                // Check for PUBLIC or SYSTEM
                if self.input.starts_with_ignore_case("PUBLIC") {
                    self.input.skip(6);
                    self.state = State::AfterDoctypePublicKeyword;
                } else if self.input.starts_with_ignore_case("SYSTEM") {
                    self.input.skip(6);
                    self.state = State::AfterDoctypeSystemKeyword;
                } else {
                    self.emit_error("invalid-character-sequence-after-doctype-name");
                    self.current_doctype.force_quirks = true;
                    self.reconsume = true;
                    self.state = State::BogusDoctype;
                }
                None
            }
        }
    }

    fn bogus_doctype_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('>') => {
                self.state = State::Data;
                self.emit_doctype()
            }
            Some('\0') => {
                self.emit_error("unexpected-null-character");
                None
            }
            None => self.emit_doctype(),
            _ => None,
        }
    }

    // Additional DOCTYPE states (simplified)
    fn after_doctype_public_keyword_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                self.state = State::BeforeDoctypePublicIdentifier;
                None
            }
            Some('"') => {
                self.emit_error("missing-whitespace-after-doctype-public-keyword");
                self.current_doctype.public_id = Some(CompactString::new(""));
                self.state = State::DoctypePublicIdentifierDoubleQuoted;
                None
            }
            Some('\'') => {
                self.emit_error("missing-whitespace-after-doctype-public-keyword");
                self.current_doctype.public_id = Some(CompactString::new(""));
                self.state = State::DoctypePublicIdentifierSingleQuoted;
                None
            }
            Some('>') => {
                self.emit_error("missing-doctype-public-identifier");
                self.current_doctype.force_quirks = true;
                self.state = State::Data;
                self.emit_doctype()
            }
            None => {
                self.emit_error("eof-in-doctype");
                self.current_doctype.force_quirks = true;
                self.emit_doctype()
            }
            _ => {
                self.emit_error("missing-quote-before-doctype-public-identifier");
                self.current_doctype.force_quirks = true;
                self.reconsume = true;
                self.state = State::BogusDoctype;
                None
            }
        }
    }

    fn after_doctype_system_keyword_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                self.state = State::BeforeDoctypeSystemIdentifier;
                None
            }
            Some('"') => {
                self.emit_error("missing-whitespace-after-doctype-system-keyword");
                self.current_doctype.system_id = Some(CompactString::new(""));
                self.state = State::DoctypeSystemIdentifierDoubleQuoted;
                None
            }
            Some('\'') => {
                self.emit_error("missing-whitespace-after-doctype-system-keyword");
                self.current_doctype.system_id = Some(CompactString::new(""));
                self.state = State::DoctypeSystemIdentifierSingleQuoted;
                None
            }
            Some('>') => {
                self.emit_error("missing-doctype-system-identifier");
                self.current_doctype.force_quirks = true;
                self.state = State::Data;
                self.emit_doctype()
            }
            None => {
                self.emit_error("eof-in-doctype");
                self.current_doctype.force_quirks = true;
                self.emit_doctype()
            }
            _ => {
                self.emit_error("missing-quote-before-doctype-system-identifier");
                self.current_doctype.force_quirks = true;
                self.reconsume = true;
                self.state = State::BogusDoctype;
                None
            }
        }
    }

    fn before_doctype_public_identifier_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => None,
            Some('"') => {
                self.current_doctype.public_id = Some(CompactString::new(""));
                self.state = State::DoctypePublicIdentifierDoubleQuoted;
                None
            }
            Some('\'') => {
                self.current_doctype.public_id = Some(CompactString::new(""));
                self.state = State::DoctypePublicIdentifierSingleQuoted;
                None
            }
            Some('>') => {
                self.emit_error("missing-doctype-public-identifier");
                self.current_doctype.force_quirks = true;
                self.state = State::Data;
                self.emit_doctype()
            }
            None => {
                self.emit_error("eof-in-doctype");
                self.current_doctype.force_quirks = true;
                self.emit_doctype()
            }
            _ => {
                self.emit_error("missing-quote-before-doctype-public-identifier");
                self.current_doctype.force_quirks = true;
                self.reconsume = true;
                self.state = State::BogusDoctype;
                None
            }
        }
    }

    fn doctype_public_identifier_double_quoted_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('"') => {
                self.state = State::AfterDoctypePublicIdentifier;
                None
            }
            Some('\0') => {
                self.emit_error("unexpected-null-character");
                if let Some(ref mut id) = self.current_doctype.public_id {
                    id.push('\u{FFFD}');
                }
                None
            }
            Some('>') => {
                self.emit_error("abrupt-doctype-public-identifier");
                self.current_doctype.force_quirks = true;
                self.state = State::Data;
                self.emit_doctype()
            }
            Some(c) => {
                if let Some(ref mut id) = self.current_doctype.public_id {
                    id.push(c);
                }
                None
            }
            None => {
                self.emit_error("eof-in-doctype");
                self.current_doctype.force_quirks = true;
                self.emit_doctype()
            }
        }
    }

    fn doctype_public_identifier_single_quoted_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('\'') => {
                self.state = State::AfterDoctypePublicIdentifier;
                None
            }
            Some('\0') => {
                self.emit_error("unexpected-null-character");
                if let Some(ref mut id) = self.current_doctype.public_id {
                    id.push('\u{FFFD}');
                }
                None
            }
            Some('>') => {
                self.emit_error("abrupt-doctype-public-identifier");
                self.current_doctype.force_quirks = true;
                self.state = State::Data;
                self.emit_doctype()
            }
            Some(c) => {
                if let Some(ref mut id) = self.current_doctype.public_id {
                    id.push(c);
                }
                None
            }
            None => {
                self.emit_error("eof-in-doctype");
                self.current_doctype.force_quirks = true;
                self.emit_doctype()
            }
        }
    }

    fn after_doctype_public_identifier_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                self.state = State::BetweenDoctypePublicAndSystemIdentifiers;
                None
            }
            Some('>') => {
                self.state = State::Data;
                self.emit_doctype()
            }
            Some('"') => {
                self.emit_error("missing-whitespace-between-doctype-public-and-system-identifiers");
                self.current_doctype.system_id = Some(CompactString::new(""));
                self.state = State::DoctypeSystemIdentifierDoubleQuoted;
                None
            }
            Some('\'') => {
                self.emit_error("missing-whitespace-between-doctype-public-and-system-identifiers");
                self.current_doctype.system_id = Some(CompactString::new(""));
                self.state = State::DoctypeSystemIdentifierSingleQuoted;
                None
            }
            None => {
                self.emit_error("eof-in-doctype");
                self.current_doctype.force_quirks = true;
                self.emit_doctype()
            }
            _ => {
                self.emit_error("missing-quote-before-doctype-system-identifier");
                self.current_doctype.force_quirks = true;
                self.reconsume = true;
                self.state = State::BogusDoctype;
                None
            }
        }
    }

    fn between_doctype_public_and_system_identifiers_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => None,
            Some('>') => {
                self.state = State::Data;
                self.emit_doctype()
            }
            Some('"') => {
                self.current_doctype.system_id = Some(CompactString::new(""));
                self.state = State::DoctypeSystemIdentifierDoubleQuoted;
                None
            }
            Some('\'') => {
                self.current_doctype.system_id = Some(CompactString::new(""));
                self.state = State::DoctypeSystemIdentifierSingleQuoted;
                None
            }
            None => {
                self.emit_error("eof-in-doctype");
                self.current_doctype.force_quirks = true;
                self.emit_doctype()
            }
            _ => {
                self.emit_error("missing-quote-before-doctype-system-identifier");
                self.current_doctype.force_quirks = true;
                self.reconsume = true;
                self.state = State::BogusDoctype;
                None
            }
        }
    }

    fn before_doctype_system_identifier_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => None,
            Some('"') => {
                self.current_doctype.system_id = Some(CompactString::new(""));
                self.state = State::DoctypeSystemIdentifierDoubleQuoted;
                None
            }
            Some('\'') => {
                self.current_doctype.system_id = Some(CompactString::new(""));
                self.state = State::DoctypeSystemIdentifierSingleQuoted;
                None
            }
            Some('>') => {
                self.emit_error("missing-doctype-system-identifier");
                self.current_doctype.force_quirks = true;
                self.state = State::Data;
                self.emit_doctype()
            }
            None => {
                self.emit_error("eof-in-doctype");
                self.current_doctype.force_quirks = true;
                self.emit_doctype()
            }
            _ => {
                self.emit_error("missing-quote-before-doctype-system-identifier");
                self.current_doctype.force_quirks = true;
                self.reconsume = true;
                self.state = State::BogusDoctype;
                None
            }
        }
    }

    fn doctype_system_identifier_double_quoted_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('"') => {
                self.state = State::AfterDoctypeSystemIdentifier;
                None
            }
            Some('\0') => {
                self.emit_error("unexpected-null-character");
                if let Some(ref mut id) = self.current_doctype.system_id {
                    id.push('\u{FFFD}');
                }
                None
            }
            Some('>') => {
                self.emit_error("abrupt-doctype-system-identifier");
                self.current_doctype.force_quirks = true;
                self.state = State::Data;
                self.emit_doctype()
            }
            Some(c) => {
                if let Some(ref mut id) = self.current_doctype.system_id {
                    id.push(c);
                }
                None
            }
            None => {
                self.emit_error("eof-in-doctype");
                self.current_doctype.force_quirks = true;
                self.emit_doctype()
            }
        }
    }

    fn doctype_system_identifier_single_quoted_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('\'') => {
                self.state = State::AfterDoctypeSystemIdentifier;
                None
            }
            Some('\0') => {
                self.emit_error("unexpected-null-character");
                if let Some(ref mut id) = self.current_doctype.system_id {
                    id.push('\u{FFFD}');
                }
                None
            }
            Some('>') => {
                self.emit_error("abrupt-doctype-system-identifier");
                self.current_doctype.force_quirks = true;
                self.state = State::Data;
                self.emit_doctype()
            }
            Some(c) => {
                if let Some(ref mut id) = self.current_doctype.system_id {
                    id.push(c);
                }
                None
            }
            None => {
                self.emit_error("eof-in-doctype");
                self.current_doctype.force_quirks = true;
                self.emit_doctype()
            }
        }
    }

    fn after_doctype_system_identifier_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => None,
            Some('>') => {
                self.state = State::Data;
                self.emit_doctype()
            }
            None => {
                self.emit_error("eof-in-doctype");
                self.current_doctype.force_quirks = true;
                self.emit_doctype()
            }
            _ => {
                self.emit_error("unexpected-character-after-doctype-system-identifier");
                self.reconsume = true;
                self.state = State::BogusDoctype;
                None
            }
        }
    }

    fn cdata_section_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some(']') => {
                self.state = State::CdataSectionBracket;
                None
            }
            Some(c) => {
                self.text_buffer.push(c);
                None
            }
            None => {
                self.emit_error("eof-in-cdata");
                self.flush_text()
            }
        }
    }

    fn cdata_section_bracket_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some(']') => {
                self.state = State::CdataSectionEnd;
                None
            }
            _ => {
                self.text_buffer.push(']');
                self.reconsume = true;
                self.state = State::CdataSection;
                None
            }
        }
    }

    fn cdata_section_end_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some(']') => {
                self.text_buffer.push(']');
                None
            }
            Some('>') => {
                self.state = State::Data;
                None
            }
            _ => {
                self.text_buffer.push_str("]]");
                self.reconsume = true;
                self.state = State::CdataSection;
                None
            }
        }
    }

    fn rcdata_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('&') => {
                self.text_buffer.push('&');
                None
            }
            Some('<') => {
                self.state = State::RcDataLessThanSign;
                None
            }
            Some('\0') => {
                self.emit_error("unexpected-null-character");
                self.text_buffer.push('\u{FFFD}');
                None
            }
            Some(c) => {
                self.text_buffer.push(c);
                None
            }
            None => self.flush_text(),
        }
    }

    fn rcdata_less_than_sign_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('/') => {
                self.temp_buffer.clear();
                self.state = State::RcDataEndTagOpen;
                None
            }
            _ => {
                self.text_buffer.push('<');
                self.reconsume = true;
                self.state = State::RcData;
                None
            }
        }
    }

    fn rcdata_end_tag_open_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some(c) if c.is_ascii_alphabetic() => {
                self.start_tag(TagKind::End);
                self.reconsume = true;
                self.state = State::RcDataEndTagName;
                None
            }
            _ => {
                self.text_buffer.push_str("</");
                self.reconsume = true;
                self.state = State::RcData;
                None
            }
        }
    }

    fn rcdata_end_tag_name_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                if self.is_appropriate_end_tag() {
                    self.state = State::BeforeAttributeName;
                } else {
                    self.text_buffer.push_str("</");
                    self.text_buffer.push_str(&self.temp_buffer);
                    self.reconsume = true;
                    self.state = State::RcData;
                }
                None
            }
            Some('/') => {
                if self.is_appropriate_end_tag() {
                    self.state = State::SelfClosingStartTag;
                } else {
                    self.text_buffer.push_str("</");
                    self.text_buffer.push_str(&self.temp_buffer);
                    self.reconsume = true;
                    self.state = State::RcData;
                }
                None
            }
            Some('>') => {
                if self.is_appropriate_end_tag() {
                    self.state = State::Data;
                    let token = self.flush_text();
                    let tag_token = self.emit_tag().unwrap_or(Token::EOF);
                    if token.is_some() {
                        self.pending_tokens.push(tag_token);
                        return token;
                    }
                    return Some(tag_token);
                } else {
                    self.text_buffer.push_str("</");
                    self.text_buffer.push_str(&self.temp_buffer);
                    self.reconsume = true;
                    self.state = State::RcData;
                }
                None
            }
            Some(c) if c.is_ascii_alphabetic() => {
                if let Some(ref mut tag) = self.current_tag {
                    tag.name.push(c.to_ascii_lowercase());
                }
                self.temp_buffer.push(c);
                None
            }
            _ => {
                self.text_buffer.push_str("</");
                self.text_buffer.push_str(&self.temp_buffer);
                self.reconsume = true;
                self.state = State::RcData;
                None
            }
        }
    }

    fn rawtext_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('<') => {
                self.state = State::RawTextLessThanSign;
                None
            }
            Some('\0') => {
                self.emit_error("unexpected-null-character");
                self.text_buffer.push('\u{FFFD}');
                None
            }
            Some(c) => {
                self.text_buffer.push(c);
                None
            }
            None => self.flush_text(),
        }
    }

    fn rawtext_less_than_sign_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('/') => {
                self.temp_buffer.clear();
                self.state = State::RawTextEndTagOpen;
                None
            }
            _ => {
                self.text_buffer.push('<');
                self.reconsume = true;
                self.state = State::RawText;
                None
            }
        }
    }

    fn rawtext_end_tag_open_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some(c) if c.is_ascii_alphabetic() => {
                self.start_tag(TagKind::End);
                self.reconsume = true;
                self.state = State::RawTextEndTagName;
                None
            }
            _ => {
                self.text_buffer.push_str("</");
                self.reconsume = true;
                self.state = State::RawText;
                None
            }
        }
    }

    fn rawtext_end_tag_name_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                if self.is_appropriate_end_tag() {
                    self.state = State::BeforeAttributeName;
                } else {
                    self.text_buffer.push_str("</");
                    self.text_buffer.push_str(&self.temp_buffer);
                    self.reconsume = true;
                    self.state = State::RawText;
                }
                None
            }
            Some('/') => {
                if self.is_appropriate_end_tag() {
                    self.state = State::SelfClosingStartTag;
                } else {
                    self.text_buffer.push_str("</");
                    self.text_buffer.push_str(&self.temp_buffer);
                    self.reconsume = true;
                    self.state = State::RawText;
                }
                None
            }
            Some('>') => {
                if self.is_appropriate_end_tag() {
                    self.state = State::Data;
                    let token = self.flush_text();
                    let tag_token = self.emit_tag().unwrap_or(Token::EOF);
                    if token.is_some() {
                        self.pending_tokens.push(tag_token);
                        return token;
                    }
                    return Some(tag_token);
                } else {
                    self.text_buffer.push_str("</");
                    self.text_buffer.push_str(&self.temp_buffer);
                    self.reconsume = true;
                    self.state = State::RawText;
                }
                None
            }
            Some(c) if c.is_ascii_alphabetic() => {
                if let Some(ref mut tag) = self.current_tag {
                    tag.name.push(c.to_ascii_lowercase());
                }
                self.temp_buffer.push(c);
                None
            }
            _ => {
                self.text_buffer.push_str("</");
                self.text_buffer.push_str(&self.temp_buffer);
                self.reconsume = true;
                self.state = State::RawText;
                None
            }
        }
    }

    fn plaintext_state(&mut self) -> Option<Token> {
        match self.current_char {
            Some('\0') => {
                self.emit_error("unexpected-null-character");
                self.text_buffer.push('\u{FFFD}');
                None
            }
            Some(c) => {
                self.text_buffer.push(c);
                None
            }
            None => self.flush_text(),
        }
    }

    /// Add the current attribute to the tag.
    fn add_current_attribute(&mut self) {
        if !self.current_attr_name.is_empty() {
            if let Some(ref mut tag) = self.current_tag {
                // Check for duplicates
                let name = std::mem::take(&mut self.current_attr_name);
                let value = if self.current_attr_value.is_empty() {
                    None
                } else {
                    Some(std::mem::take(&mut self.current_attr_value).into())
                };

                // Only add if not duplicate
                if !tag.has_attr(&name) {
                    tag.attrs.push((name.into(), value));
                } else if self.collect_errors {
                    self.emit_error("duplicate-attribute");
                }
            }
        }
        self.current_attr_name.clear();
        self.current_attr_value.clear();
        self.current_attr_value_has_amp = false;
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.next_token()?;
        if matches!(token, Token::EOF) {
            None
        } else {
            Some(token)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tag() {
        let mut tokenizer = Tokenizer::new("<div>");
        let token = tokenizer.next_token().unwrap();
        assert!(matches!(token, Token::Tag(ref t) if t.name == "div" && t.is_start()));
    }

    #[test]
    fn test_tag_with_attributes() {
        let mut tokenizer = Tokenizer::new(r#"<div class="container" id="main">"#);
        let token = tokenizer.next_token().unwrap();
        if let Token::Tag(tag) = token {
            assert_eq!(tag.name.as_str(), "div");
            assert_eq!(tag.get_attr("class"), Some(Some("container")));
            assert_eq!(tag.get_attr("id"), Some(Some("main")));
        } else {
            panic!("Expected tag token");
        }
    }

    #[test]
    fn test_self_closing_tag() {
        let mut tokenizer = Tokenizer::new("<br/>");
        let token = tokenizer.next_token().unwrap();
        if let Token::Tag(tag) = token {
            assert_eq!(tag.name.as_str(), "br");
            assert!(tag.self_closing);
        } else {
            panic!("Expected tag token");
        }
    }

    #[test]
    fn test_comment() {
        let mut tokenizer = Tokenizer::new("<!-- comment -->");
        let token = tokenizer.next_token().unwrap();
        if let Token::Comment(comment) = token {
            assert_eq!(comment.data.as_str(), " comment ");
        } else {
            panic!("Expected comment token");
        }
    }

    #[test]
    fn test_doctype() {
        let mut tokenizer = Tokenizer::new("<!DOCTYPE html>");
        let token = tokenizer.next_token().unwrap();
        if let Token::Doctype(doctype) = token {
            assert_eq!(doctype.doctype.name.as_deref(), Some("html"));
            assert!(!doctype.doctype.force_quirks);
        } else {
            panic!("Expected doctype token");
        }
    }

    #[test]
    fn test_text_content() {
        let mut tokenizer = Tokenizer::new("Hello World");
        let token = tokenizer.next_token().unwrap();
        if let Token::Characters(chars) = token {
            assert_eq!(chars.data.as_str(), "Hello World");
        } else {
            panic!("Expected character token");
        }
    }

    #[test]
    fn test_mixed_content() {
        let mut tokenizer = Tokenizer::new("<p>Hello</p>");
        let tokens: Vec<Token> = tokenizer.collect();

        assert_eq!(tokens.len(), 3);
        assert!(matches!(&tokens[0], Token::Tag(t) if t.name == "p" && t.is_start()));
        assert!(matches!(&tokens[1], Token::Characters(c) if c.data == "Hello"));
        assert!(matches!(&tokens[2], Token::Tag(t) if t.name == "p" && t.is_end()));
    }
}

//! HTML5 tree builder implementation.
//!
//! Implements the WHATWG HTML5 tree construction algorithm.

mod constants;

pub use constants::*;

use compact_str::CompactString;

use crate::dom::{Dom, Element, Namespace, NodeId, NodeKind};
use crate::error::ParseError;
use crate::tokenizer::{CommentToken, Doctype, Tag, Token, Tokenizer};

/// Tree builder insertion mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertionMode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    InHeadNoscript,
    AfterHead,
    InBody,
    Text,
    InTable,
    InTableText,
    InCaption,
    InColumnGroup,
    InTableBody,
    InRow,
    InCell,
    InSelect,
    InSelectInTable,
    InTemplate,
    AfterBody,
    InFrameset,
    AfterFrameset,
    AfterAfterBody,
    AfterAfterFrameset,
}

/// A marker or element in the active formatting elements list.
#[derive(Debug, Clone)]
pub enum FormattingEntry {
    /// A marker (scope boundary).
    Marker,
    /// An element with its attributes.
    Element {
        node_id: NodeId,
        name: CompactString,
        attrs: Vec<(CompactString, Option<CompactString>)>,
    },
}

/// HTML5 tree builder.
pub struct TreeBuilder<'a> {
    /// The DOM being constructed.
    pub dom: Dom,
    /// Document node ID.
    document: NodeId,
    /// Current insertion mode.
    mode: InsertionMode,
    /// Original insertion mode (for text mode).
    original_mode: Option<InsertionMode>,
    /// Stack of open elements.
    open_elements: Vec<NodeId>,
    /// Active formatting elements.
    active_formatting: Vec<FormattingEntry>,
    /// Head element pointer.
    head_element: Option<NodeId>,
    /// Form element pointer.
    form_element: Option<NodeId>,
    /// Whether scripting is enabled.
    scripting: bool,
    /// Frameset-ok flag.
    frameset_ok: bool,
    /// Template insertion modes stack.
    template_modes: Vec<InsertionMode>,
    /// Foster parenting flag.
    foster_parenting: bool,
    /// Collected parse errors.
    pub errors: Vec<ParseError>,
    /// Whether to collect errors.
    collect_errors: bool,
    /// Pending table character tokens.
    pending_table_chars: Vec<char>,
    /// The tokenizer.
    tokenizer: Tokenizer<'a>,
}

impl<'a> TreeBuilder<'a> {
    /// Create a new tree builder for the given HTML.
    pub fn new(html: &'a str) -> Self {
        let (dom, document) = Dom::with_document();

        Self {
            dom,
            document,
            mode: InsertionMode::Initial,
            original_mode: None,
            open_elements: Vec::new(),
            active_formatting: Vec::new(),
            head_element: None,
            form_element: None,
            scripting: false,
            frameset_ok: true,
            template_modes: Vec::new(),
            foster_parenting: false,
            errors: Vec::new(),
            collect_errors: false,
            pending_table_chars: Vec::new(),
            tokenizer: Tokenizer::new(html),
        }
    }

    /// Enable error collection.
    pub fn with_errors(mut self) -> Self {
        self.collect_errors = true;
        self.tokenizer.collect_errors = true;
        self
    }

    /// Parse the HTML and return the document root.
    pub fn parse(mut self) -> (Dom, NodeId, Vec<ParseError>) {
        loop {
            let token = match self.tokenizer.next_token() {
                Some(t) => t,
                None => break,
            };

            if matches!(token, Token::EOF) {
                self.process_eof();
                break;
            }

            self.process_token(token);
        }

        // Collect errors from tokenizer
        let mut errors = std::mem::take(&mut self.errors);
        errors.extend(std::mem::take(&mut self.tokenizer.errors));

        (self.dom, self.document, errors)
    }

    /// Process a single token.
    fn process_token(&mut self, token: Token) {
        match self.mode {
            InsertionMode::Initial => self.process_initial(token),
            InsertionMode::BeforeHtml => self.process_before_html(token),
            InsertionMode::BeforeHead => self.process_before_head(token),
            InsertionMode::InHead => self.process_in_head(token),
            InsertionMode::AfterHead => self.process_after_head(token),
            InsertionMode::InBody => self.process_in_body(token),
            InsertionMode::Text => self.process_text(token),
            InsertionMode::AfterBody => self.process_after_body(token),
            InsertionMode::AfterAfterBody => self.process_after_after_body(token),
            _ => self.process_in_body(token), // Simplified: use InBody for unimplemented modes
        }
    }

    /// Process EOF.
    fn process_eof(&mut self) {
        // Pop all remaining open elements
        // In a full implementation, we'd check for unclosed tags
    }

    // ========================================================================
    // Insertion mode handlers
    // ========================================================================

    fn process_initial(&mut self, token: Token) {
        match &token {
            Token::Characters(chars) if chars.data.chars().all(|c| c.is_ascii_whitespace()) => {
                // Ignore whitespace
            }
            Token::Comment(comment) => {
                self.insert_comment_at(comment, self.document);
            }
            Token::Doctype(doctype) => {
                self.insert_doctype(&doctype.doctype);
                self.mode = InsertionMode::BeforeHtml;
            }
            _ => {
                // Quirks mode handling omitted for simplicity
                self.mode = InsertionMode::BeforeHtml;
                self.process_token(token);
            }
        }
    }

    fn process_before_html(&mut self, token: Token) {
        match &token {
            Token::Doctype(_) => {
                self.emit_error("unexpected-doctype");
            }
            Token::Comment(comment) => {
                self.insert_comment_at(comment, self.document);
            }
            Token::Characters(chars) if chars.data.chars().all(|c| c.is_ascii_whitespace()) => {
                // Ignore whitespace
            }
            Token::Tag(tag) if tag.is_start() && tag.name == "html" => {
                let html = self.create_element_for_token(tag, Namespace::Html);
                self.dom.append_child(self.document, html);
                self.open_elements.push(html);
                self.mode = InsertionMode::BeforeHead;
            }
            Token::Tag(tag) if tag.is_end() => {
                let name = tag.name.as_str();
                if !["head", "body", "html", "br"].contains(&name) {
                    self.emit_error("unexpected-end-tag");
                    return;
                }
                // Fall through to anything else
                self.insert_html_element();
                self.mode = InsertionMode::BeforeHead;
                self.process_token(token);
            }
            _ => {
                self.insert_html_element();
                self.mode = InsertionMode::BeforeHead;
                self.process_token(token);
            }
        }
    }

    fn process_before_head(&mut self, token: Token) {
        match &token {
            Token::Characters(chars) if chars.data.chars().all(|c| c.is_ascii_whitespace()) => {
                // Ignore whitespace
            }
            Token::Comment(comment) => {
                self.insert_comment(comment);
            }
            Token::Doctype(_) => {
                self.emit_error("unexpected-doctype");
            }
            Token::Tag(tag) if tag.is_start() && tag.name == "html" => {
                self.process_in_body(token);
            }
            Token::Tag(tag) if tag.is_start() && tag.name == "head" => {
                let head = self.create_element_for_token(tag, Namespace::Html);
                self.insert_element(head);
                self.head_element = Some(head);
                self.mode = InsertionMode::InHead;
            }
            Token::Tag(tag) if tag.is_end() => {
                let name = tag.name.as_str();
                if !["head", "body", "html", "br"].contains(&name) {
                    self.emit_error("unexpected-end-tag");
                    return;
                }
                // Fall through
                self.insert_head_element();
                self.mode = InsertionMode::InHead;
                self.process_token(token);
            }
            _ => {
                self.insert_head_element();
                self.mode = InsertionMode::InHead;
                self.process_token(token);
            }
        }
    }

    fn process_in_head(&mut self, token: Token) {
        match &token {
            Token::Characters(chars) if chars.data.chars().all(|c| c.is_ascii_whitespace()) => {
                self.insert_text(&chars.data);
            }
            Token::Comment(comment) => {
                self.insert_comment(comment);
            }
            Token::Doctype(_) => {
                self.emit_error("unexpected-doctype");
            }
            Token::Tag(tag) if tag.is_start() => {
                let name = tag.name.as_str();
                match name {
                    "html" => self.process_in_body(token),
                    "base" | "basefont" | "bgsound" | "link" => {
                        let elem = self.create_element_for_token(tag, Namespace::Html);
                        self.insert_element(elem);
                        self.open_elements.pop();
                    }
                    "meta" => {
                        let elem = self.create_element_for_token(tag, Namespace::Html);
                        self.insert_element(elem);
                        self.open_elements.pop();
                    }
                    "title" => {
                        self.parse_raw_text_or_rcdata(tag, true);
                    }
                    "noscript" if !self.scripting => {
                        let elem = self.create_element_for_token(tag, Namespace::Html);
                        self.insert_element(elem);
                        self.mode = InsertionMode::InHeadNoscript;
                    }
                    "noframes" | "style" => {
                        self.parse_raw_text_or_rcdata(tag, false);
                    }
                    "script" => {
                        let elem = self.create_element_for_token(tag, Namespace::Html);
                        self.insert_element(elem);
                        self.original_mode = Some(self.mode);
                        self.mode = InsertionMode::Text;
                    }
                    "head" => {
                        self.emit_error("unexpected-start-tag");
                    }
                    "template" => {
                        let elem = self.create_element_for_token(tag, Namespace::Html);
                        self.insert_element(elem);
                        self.active_formatting.push(FormattingEntry::Marker);
                        self.frameset_ok = false;
                        self.mode = InsertionMode::InTemplate;
                        self.template_modes.push(InsertionMode::InTemplate);
                    }
                    _ => {
                        self.open_elements.pop(); // Pop head
                        self.mode = InsertionMode::AfterHead;
                        self.process_token(token);
                    }
                }
            }
            Token::Tag(tag) if tag.is_end() => {
                let name = tag.name.as_str();
                match name {
                    "head" => {
                        self.open_elements.pop();
                        self.mode = InsertionMode::AfterHead;
                    }
                    "body" | "html" | "br" => {
                        self.open_elements.pop(); // Pop head
                        self.mode = InsertionMode::AfterHead;
                        self.process_token(token);
                    }
                    "template" => {
                        // Handle template end tag
                        if !self.template_modes.is_empty() {
                            self.template_modes.pop();
                        }
                        self.clear_active_formatting_to_marker();
                        // Pop elements until template
                        while let Some(id) = self.open_elements.pop() {
                            let node = self.dom.get(id);
                            if node.kind.name() == "template" {
                                break;
                            }
                        }
                        self.reset_insertion_mode();
                    }
                    _ => {
                        self.emit_error("unexpected-end-tag");
                    }
                }
            }
            _ => {
                self.open_elements.pop(); // Pop head
                self.mode = InsertionMode::AfterHead;
                self.process_token(token);
            }
        }
    }

    fn process_after_head(&mut self, token: Token) {
        match &token {
            Token::Characters(chars) if chars.data.chars().all(|c| c.is_ascii_whitespace()) => {
                self.insert_text(&chars.data);
            }
            Token::Comment(comment) => {
                self.insert_comment(comment);
            }
            Token::Doctype(_) => {
                self.emit_error("unexpected-doctype");
            }
            Token::Tag(tag) if tag.is_start() => {
                let name = tag.name.as_str();
                match name {
                    "html" => self.process_in_body(token),
                    "body" => {
                        let body = self.create_element_for_token(tag, Namespace::Html);
                        self.insert_element(body);
                        self.frameset_ok = false;
                        self.mode = InsertionMode::InBody;
                    }
                    "frameset" => {
                        let elem = self.create_element_for_token(tag, Namespace::Html);
                        self.insert_element(elem);
                        self.mode = InsertionMode::InFrameset;
                    }
                    "base" | "basefont" | "bgsound" | "link" | "meta" | "noframes" | "script"
                    | "style" | "template" | "title" => {
                        self.emit_error("unexpected-start-tag");
                        if let Some(head) = self.head_element {
                            self.open_elements.push(head);
                            self.process_in_head(token);
                            // Remove head from stack
                            if let Some(pos) =
                                self.open_elements.iter().position(|&id| id == head)
                            {
                                self.open_elements.remove(pos);
                            }
                        }
                    }
                    "head" => {
                        self.emit_error("unexpected-start-tag");
                    }
                    _ => {
                        self.insert_body_element();
                        self.mode = InsertionMode::InBody;
                        self.process_token(token);
                    }
                }
            }
            Token::Tag(tag) if tag.is_end() => {
                let name = tag.name.as_str();
                match name {
                    "template" => self.process_in_head(token),
                    "body" | "html" | "br" => {
                        self.insert_body_element();
                        self.mode = InsertionMode::InBody;
                        self.process_token(token);
                    }
                    _ => {
                        self.emit_error("unexpected-end-tag");
                    }
                }
            }
            _ => {
                self.insert_body_element();
                self.mode = InsertionMode::InBody;
                self.process_token(token);
            }
        }
    }

    fn process_in_body(&mut self, token: Token) {
        match token {
            Token::Characters(ref chars) => {
                self.reconstruct_active_formatting();
                self.insert_text(&chars.data);
                if chars.data.chars().any(|c| !c.is_ascii_whitespace()) {
                    self.frameset_ok = false;
                }
            }
            Token::Comment(ref comment) => {
                self.insert_comment(comment);
            }
            Token::Doctype(_) => {
                self.emit_error("unexpected-doctype");
            }
            Token::Tag(ref tag) if tag.is_start() => {
                self.process_in_body_start_tag(tag.clone());
            }
            Token::Tag(ref tag) if tag.is_end() => {
                self.process_in_body_end_tag(tag.clone());
            }
            Token::EOF => {
                // Handle EOF in body
            }
            _ => {}
        }
    }

    fn process_in_body_start_tag(&mut self, tag: Tag) {
        let name = tag.name.as_str();

        match name {
            "html" => {
                self.emit_error("unexpected-start-tag");
                // Merge attributes into html element
                if let Some(&html_id) = self.open_elements.first() {
                    if let NodeKind::Element(ref mut el) =
                        &mut self.dom.get_mut(html_id).kind
                    {
                        for (attr_name, attr_value) in &tag.attrs {
                            if !el.attrs.contains(attr_name) {
                                el.attrs.set(attr_name.clone(), attr_value.clone());
                            }
                        }
                    }
                }
            }
            "base" | "basefont" | "bgsound" | "link" | "meta" | "noframes" | "script"
            | "style" | "template" | "title" => {
                self.process_in_head(Token::Tag(tag));
            }
            "body" => {
                self.emit_error("unexpected-start-tag");
                // Merge attributes
            }
            "frameset" => {
                self.emit_error("unexpected-start-tag");
            }
            "address" | "article" | "aside" | "blockquote" | "center" | "details" | "dialog"
            | "dir" | "div" | "dl" | "fieldset" | "figcaption" | "figure" | "footer" | "header"
            | "hgroup" | "main" | "menu" | "nav" | "ol" | "p" | "search" | "section"
            | "summary" | "ul" => {
                if self.has_element_in_button_scope("p") {
                    self.close_p_element();
                }
                let elem = self.create_element_for_token(&tag, Namespace::Html);
                self.insert_element(elem);
            }
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                if self.has_element_in_button_scope("p") {
                    self.close_p_element();
                }
                // Check if current element is a heading
                if let Some(&current) = self.open_elements.last() {
                    let current_name = self.dom.get(current).kind.name();
                    if HEADING_ELEMENTS.contains(current_name) {
                        self.emit_error("unexpected-start-tag");
                        self.open_elements.pop();
                    }
                }
                let elem = self.create_element_for_token(&tag, Namespace::Html);
                self.insert_element(elem);
            }
            "pre" | "listing" => {
                if self.has_element_in_button_scope("p") {
                    self.close_p_element();
                }
                let elem = self.create_element_for_token(&tag, Namespace::Html);
                self.insert_element(elem);
                self.frameset_ok = false;
            }
            "form" => {
                if self.form_element.is_some() && !self.has_template_in_stack() {
                    self.emit_error("unexpected-start-tag");
                } else {
                    if self.has_element_in_button_scope("p") {
                        self.close_p_element();
                    }
                    let elem = self.create_element_for_token(&tag, Namespace::Html);
                    self.insert_element(elem);
                    if !self.has_template_in_stack() {
                        self.form_element = Some(elem);
                    }
                }
            }
            "li" => {
                self.frameset_ok = false;
                // Close any previous li
                for i in (0..self.open_elements.len()).rev() {
                    let id = self.open_elements[i];
                    let node_name = self.dom.get(id).kind.name();
                    if node_name == "li" {
                        self.generate_implied_end_tags_except("li");
                        self.pop_until("li");
                        break;
                    }
                    if SPECIAL_ELEMENTS.contains(node_name)
                        && !["address", "div", "p"].contains(&node_name)
                    {
                        break;
                    }
                }
                if self.has_element_in_button_scope("p") {
                    self.close_p_element();
                }
                let elem = self.create_element_for_token(&tag, Namespace::Html);
                self.insert_element(elem);
            }
            "dd" | "dt" => {
                self.frameset_ok = false;
                if self.has_element_in_button_scope("p") {
                    self.close_p_element();
                }
                let elem = self.create_element_for_token(&tag, Namespace::Html);
                self.insert_element(elem);
            }
            "plaintext" => {
                if self.has_element_in_button_scope("p") {
                    self.close_p_element();
                }
                let elem = self.create_element_for_token(&tag, Namespace::Html);
                self.insert_element(elem);
                // Switch tokenizer to plaintext state (not implemented)
            }
            "button" => {
                if self.has_element_in_scope("button") {
                    self.emit_error("unexpected-start-tag");
                    self.generate_implied_end_tags();
                    self.pop_until("button");
                }
                self.reconstruct_active_formatting();
                let elem = self.create_element_for_token(&tag, Namespace::Html);
                self.insert_element(elem);
                self.frameset_ok = false;
            }
            "a" => {
                // Check for existing a in active formatting
                self.reconstruct_active_formatting();
                let elem = self.create_element_for_token(&tag, Namespace::Html);
                self.insert_element(elem);
                self.push_active_formatting(elem, &tag);
            }
            "b" | "big" | "code" | "em" | "font" | "i" | "s" | "small" | "strike" | "strong"
            | "tt" | "u" => {
                self.reconstruct_active_formatting();
                let elem = self.create_element_for_token(&tag, Namespace::Html);
                self.insert_element(elem);
                self.push_active_formatting(elem, &tag);
            }
            "nobr" => {
                self.reconstruct_active_formatting();
                if self.has_element_in_scope("nobr") {
                    self.emit_error("unexpected-start-tag");
                    // Run adoption agency for nobr
                    self.pop_until("nobr");
                    self.reconstruct_active_formatting();
                }
                let elem = self.create_element_for_token(&tag, Namespace::Html);
                self.insert_element(elem);
                self.push_active_formatting(elem, &tag);
            }
            "table" => {
                if self.has_element_in_button_scope("p") {
                    self.close_p_element();
                }
                let elem = self.create_element_for_token(&tag, Namespace::Html);
                self.insert_element(elem);
                self.frameset_ok = false;
                self.mode = InsertionMode::InTable;
            }
            "area" | "br" | "embed" | "img" | "keygen" | "wbr" => {
                self.reconstruct_active_formatting();
                let elem = self.create_element_for_token(&tag, Namespace::Html);
                self.insert_element(elem);
                self.open_elements.pop();
                self.frameset_ok = false;
            }
            "input" => {
                self.reconstruct_active_formatting();
                let elem = self.create_element_for_token(&tag, Namespace::Html);
                self.insert_element(elem);
                self.open_elements.pop();
                // Check type attribute for frameset-ok
                let is_hidden = tag
                    .get_attr("type")
                    .and_then(|v| v)
                    .map(|v| v.eq_ignore_ascii_case("hidden"))
                    .unwrap_or(false);
                if !is_hidden {
                    self.frameset_ok = false;
                }
            }
            "hr" => {
                if self.has_element_in_button_scope("p") {
                    self.close_p_element();
                }
                let elem = self.create_element_for_token(&tag, Namespace::Html);
                self.insert_element(elem);
                self.open_elements.pop();
                self.frameset_ok = false;
            }
            "image" => {
                self.emit_error("unexpected-start-tag");
                // Treat as img
                let mut img_tag = tag;
                img_tag.name = "img".into();
                self.process_in_body_start_tag(img_tag);
            }
            "textarea" => {
                let elem = self.create_element_for_token(&tag, Namespace::Html);
                self.insert_element(elem);
                self.frameset_ok = false;
                self.original_mode = Some(self.mode);
                self.mode = InsertionMode::Text;
            }
            "select" => {
                self.reconstruct_active_formatting();
                let elem = self.create_element_for_token(&tag, Namespace::Html);
                self.insert_element(elem);
                self.frameset_ok = false;
                self.mode = InsertionMode::InSelect;
            }
            "optgroup" | "option" => {
                if let Some(&current) = self.open_elements.last() {
                    if self.dom.get(current).kind.name() == "option" {
                        self.open_elements.pop();
                    }
                }
                self.reconstruct_active_formatting();
                let elem = self.create_element_for_token(&tag, Namespace::Html);
                self.insert_element(elem);
            }
            "span" => {
                self.reconstruct_active_formatting();
                let elem = self.create_element_for_token(&tag, Namespace::Html);
                self.insert_element(elem);
            }
            _ => {
                self.reconstruct_active_formatting();
                let elem = self.create_element_for_token(&tag, Namespace::Html);
                self.insert_element(elem);
            }
        }
    }

    fn process_in_body_end_tag(&mut self, tag: Tag) {
        let name = tag.name.as_str();

        match name {
            "template" => {
                self.process_in_head(Token::Tag(tag));
            }
            "body" => {
                if !self.has_element_in_scope("body") {
                    self.emit_error("unexpected-end-tag");
                    return;
                }
                self.mode = InsertionMode::AfterBody;
            }
            "html" => {
                if !self.has_element_in_scope("body") {
                    self.emit_error("unexpected-end-tag");
                    return;
                }
                self.mode = InsertionMode::AfterBody;
                self.process_token(Token::Tag(tag));
            }
            "address" | "article" | "aside" | "blockquote" | "button" | "center" | "details"
            | "dialog" | "dir" | "div" | "dl" | "fieldset" | "figcaption" | "figure"
            | "footer" | "header" | "hgroup" | "listing" | "main" | "menu" | "nav" | "ol"
            | "pre" | "search" | "section" | "summary" | "ul" => {
                if !self.has_element_in_scope(name) {
                    self.emit_error("unexpected-end-tag");
                    return;
                }
                self.generate_implied_end_tags();
                self.pop_until(name);
            }
            "form" => {
                if !self.has_template_in_stack() {
                    let form = self.form_element.take();
                    if form.is_none() || !self.has_element_in_scope("form") {
                        self.emit_error("unexpected-end-tag");
                        return;
                    }
                    self.generate_implied_end_tags();
                    if let Some(form_id) = form {
                        // Remove form from stack
                        self.open_elements.retain(|&id| id != form_id);
                    }
                } else {
                    if !self.has_element_in_scope("form") {
                        self.emit_error("unexpected-end-tag");
                        return;
                    }
                    self.generate_implied_end_tags();
                    self.pop_until("form");
                }
            }
            "p" => {
                if !self.has_element_in_button_scope("p") {
                    self.emit_error("unexpected-end-tag");
                    let p = self.dom.create_element("p", Namespace::Html);
                    self.insert_element(p);
                }
                self.close_p_element();
            }
            "li" => {
                if !self.has_element_in_list_item_scope("li") {
                    self.emit_error("unexpected-end-tag");
                    return;
                }
                self.generate_implied_end_tags_except("li");
                self.pop_until("li");
            }
            "dd" | "dt" => {
                if !self.has_element_in_scope(name) {
                    self.emit_error("unexpected-end-tag");
                    return;
                }
                self.generate_implied_end_tags_except(name);
                self.pop_until(name);
            }
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                if !self.has_heading_in_scope() {
                    self.emit_error("unexpected-end-tag");
                    return;
                }
                self.generate_implied_end_tags();
                // Pop until any heading
                while let Some(id) = self.open_elements.pop() {
                    let node_name = self.dom.get(id).kind.name();
                    if HEADING_ELEMENTS.contains(node_name) {
                        break;
                    }
                }
            }
            "a" | "b" | "big" | "code" | "em" | "font" | "i" | "nobr" | "s" | "small"
            | "strike" | "strong" | "tt" | "u" => {
                // Adoption agency algorithm (simplified)
                self.run_adoption_agency(name);
            }
            "applet" | "marquee" | "object" => {
                if !self.has_element_in_scope(name) {
                    self.emit_error("unexpected-end-tag");
                    return;
                }
                self.generate_implied_end_tags();
                self.pop_until(name);
                self.clear_active_formatting_to_marker();
            }
            "br" => {
                self.emit_error("unexpected-end-tag");
                // Treat as <br>
                self.reconstruct_active_formatting();
                let br = self.dom.create_element("br", Namespace::Html);
                self.insert_element(br);
                self.open_elements.pop();
                self.frameset_ok = false;
            }
            _ => {
                // Any other end tag
                self.any_other_end_tag(name);
            }
        }
    }

    fn process_text(&mut self, token: Token) {
        match token {
            Token::Characters(chars) => {
                self.insert_text(&chars.data);
            }
            Token::EOF => {
                self.emit_error("eof-in-text");
                self.open_elements.pop();
                if let Some(mode) = self.original_mode.take() {
                    self.mode = mode;
                }
            }
            Token::Tag(ref tag) if tag.is_end() => {
                self.open_elements.pop();
                if let Some(mode) = self.original_mode.take() {
                    self.mode = mode;
                }
            }
            _ => {}
        }
    }

    fn process_after_body(&mut self, token: Token) {
        match &token {
            Token::Characters(chars) if chars.data.chars().all(|c| c.is_ascii_whitespace()) => {
                self.process_in_body(token);
            }
            Token::Comment(comment) => {
                // Insert at html element
                if let Some(&html) = self.open_elements.first() {
                    self.insert_comment_at(comment, html);
                }
            }
            Token::Doctype(_) => {
                self.emit_error("unexpected-doctype");
            }
            Token::Tag(tag) if tag.is_start() && tag.name == "html" => {
                self.process_in_body(token);
            }
            Token::Tag(tag) if tag.is_end() && tag.name == "html" => {
                self.mode = InsertionMode::AfterAfterBody;
            }
            Token::EOF => {
                // Stop parsing
            }
            _ => {
                self.emit_error("unexpected-token-after-body");
                self.mode = InsertionMode::InBody;
                self.process_token(token);
            }
        }
    }

    fn process_after_after_body(&mut self, token: Token) {
        match &token {
            Token::Comment(comment) => {
                self.insert_comment_at(comment, self.document);
            }
            Token::Doctype(_) => {
                self.process_in_body(token);
            }
            Token::Characters(chars) if chars.data.chars().all(|c| c.is_ascii_whitespace()) => {
                self.process_in_body(token);
            }
            Token::Tag(tag) if tag.is_start() && tag.name == "html" => {
                self.process_in_body(token);
            }
            Token::EOF => {
                // Stop parsing
            }
            _ => {
                self.emit_error("unexpected-token-after-body");
                self.mode = InsertionMode::InBody;
                self.process_token(token);
            }
        }
    }

    // ========================================================================
    // Helper methods
    // ========================================================================

    fn emit_error(&mut self, code: &str) {
        if self.collect_errors {
            self.errors.push(ParseError::new(code, None, None));
        }
    }

    fn create_element_for_token(&mut self, tag: &Tag, namespace: Namespace) -> NodeId {
        let mut element = Element::new(tag.name.clone(), namespace);
        for (name, value) in &tag.attrs {
            element.attrs.set(name.clone(), value.clone());
        }
        self.dom.create_node(NodeKind::Element(element))
    }

    fn insert_element(&mut self, node_id: NodeId) {
        let parent = self
            .open_elements
            .last()
            .copied()
            .unwrap_or(self.document);
        self.dom.append_child(parent, node_id);
        self.open_elements.push(node_id);
    }

    fn insert_text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }

        let parent = self
            .open_elements
            .last()
            .copied()
            .unwrap_or(self.document);

        // Try to append to existing text node
        if let Some(last_child) = self.dom.get(parent).last_child {
            if let NodeKind::Text(ref mut existing) = self.dom.get_mut(last_child).kind {
                existing.push_str(text);
                return;
            }
        }

        // Create new text node
        let text_node = self.dom.create_text(text);
        self.dom.append_child(parent, text_node);
    }

    fn insert_comment(&mut self, comment: &CommentToken) {
        let parent = self
            .open_elements
            .last()
            .copied()
            .unwrap_or(self.document);
        let comment_node = self.dom.create_comment(comment.data.clone());
        self.dom.append_child(parent, comment_node);
    }

    fn insert_comment_at(&mut self, comment: &CommentToken, parent: NodeId) {
        let comment_node = self.dom.create_comment(comment.data.clone());
        self.dom.append_child(parent, comment_node);
    }

    fn insert_doctype(&mut self, doctype: &Doctype) {
        let doctype_node = self.dom.create_node(NodeKind::Doctype(crate::dom::Doctype {
            name: doctype.name.clone(),
            public_id: doctype.public_id.clone(),
            system_id: doctype.system_id.clone(),
            force_quirks: doctype.force_quirks,
        }));
        self.dom.append_child(self.document, doctype_node);
    }

    fn insert_html_element(&mut self) {
        let html = self.dom.create_element("html", Namespace::Html);
        self.dom.append_child(self.document, html);
        self.open_elements.push(html);
    }

    fn insert_head_element(&mut self) {
        let head = self.dom.create_element("head", Namespace::Html);
        self.insert_element(head);
        self.head_element = Some(head);
    }

    fn insert_body_element(&mut self) {
        let body = self.dom.create_element("body", Namespace::Html);
        self.insert_element(body);
    }

    fn has_element_in_scope(&self, name: &str) -> bool {
        self.has_element_in_scope_with(&DEFAULT_SCOPE_TERMINATORS, name)
    }

    fn has_element_in_button_scope(&self, name: &str) -> bool {
        self.has_element_in_scope_with(&BUTTON_SCOPE_TERMINATORS, name)
    }

    fn has_element_in_list_item_scope(&self, name: &str) -> bool {
        self.has_element_in_scope_with(&LIST_ITEM_SCOPE_TERMINATORS, name)
    }

    fn has_element_in_scope_with(
        &self,
        terminators: &HashSet<&'static str>,
        name: &str,
    ) -> bool {
        for &id in self.open_elements.iter().rev() {
            let node_name = self.dom.get(id).kind.name();
            if node_name == name {
                return true;
            }
            if terminators.contains(node_name) {
                return false;
            }
        }
        false
    }

    fn has_heading_in_scope(&self) -> bool {
        for &id in self.open_elements.iter().rev() {
            let node_name = self.dom.get(id).kind.name();
            if HEADING_ELEMENTS.contains(node_name) {
                return true;
            }
            if DEFAULT_SCOPE_TERMINATORS.contains(node_name) {
                return false;
            }
        }
        false
    }

    fn has_template_in_stack(&self) -> bool {
        self.open_elements.iter().any(|&id| {
            self.dom.get(id).kind.name() == "template"
        })
    }

    fn close_p_element(&mut self) {
        self.generate_implied_end_tags_except("p");
        self.pop_until("p");
    }

    fn generate_implied_end_tags(&mut self) {
        self.generate_implied_end_tags_except("");
    }

    fn generate_implied_end_tags_except(&mut self, except: &str) {
        while let Some(&id) = self.open_elements.last() {
            let name = self.dom.get(id).kind.name();
            if name == except {
                break;
            }
            if IMPLIED_END_TAGS.contains(name) {
                self.open_elements.pop();
            } else {
                break;
            }
        }
    }

    fn pop_until(&mut self, name: &str) {
        while let Some(id) = self.open_elements.pop() {
            if self.dom.get(id).kind.name() == name {
                break;
            }
        }
    }

    fn reconstruct_active_formatting(&mut self) {
        // Simplified implementation
        if self.active_formatting.is_empty() {
            return;
        }

        // Check if last entry is a marker or in the stack
        if let Some(last) = self.active_formatting.last() {
            match last {
                FormattingEntry::Marker => return,
                FormattingEntry::Element { node_id, .. } => {
                    if self.open_elements.contains(node_id) {
                        return;
                    }
                }
            }
        }

        // Full reconstruction would be more complex
    }

    fn push_active_formatting(&mut self, node_id: NodeId, tag: &Tag) {
        let attrs: Vec<_> = tag
            .attrs
            .iter()
            .map(|(n, v)| (n.clone(), v.clone()))
            .collect();

        self.active_formatting.push(FormattingEntry::Element {
            node_id,
            name: tag.name.clone(),
            attrs,
        });
    }

    fn clear_active_formatting_to_marker(&mut self) {
        while let Some(entry) = self.active_formatting.pop() {
            if matches!(entry, FormattingEntry::Marker) {
                break;
            }
        }
    }

    fn reset_insertion_mode(&mut self) {
        // Simplified reset
        self.mode = InsertionMode::InBody;
    }

    fn run_adoption_agency(&mut self, name: &str) {
        // Simplified adoption agency - just pop until we find the element
        for i in (0..self.open_elements.len()).rev() {
            let id = self.open_elements[i];
            if self.dom.get(id).kind.name() == name {
                self.open_elements.remove(i);
                // Also remove from active formatting
                self.active_formatting.retain(|entry| {
                    if let FormattingEntry::Element { node_id, .. } = entry {
                        *node_id != id
                    } else {
                        true
                    }
                });
                break;
            }
        }
    }

    fn any_other_end_tag(&mut self, name: &str) {
        for i in (0..self.open_elements.len()).rev() {
            let id = self.open_elements[i];
            let node_name = self.dom.get(id).kind.name();

            if node_name == name {
                self.generate_implied_end_tags_except(name);
                self.open_elements.truncate(i);
                break;
            }

            if SPECIAL_ELEMENTS.contains(node_name) {
                self.emit_error("unexpected-end-tag");
                break;
            }
        }
    }

    fn parse_raw_text_or_rcdata(&mut self, tag: &Tag, _is_rcdata: bool) {
        let elem = self.create_element_for_token(tag, Namespace::Html);
        self.insert_element(elem);
        self.original_mode = Some(self.mode);
        self.mode = InsertionMode::Text;
    }
}

use std::collections::HashSet;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_document() {
        let html = "<html><head><title>Test</title></head><body><p>Hello</p></body></html>";
        let builder = TreeBuilder::new(html);
        let (dom, doc, _errors) = builder.parse();

        // Document should have children
        assert!(dom.get(doc).first_child.is_some());
    }

    #[test]
    fn test_implicit_tags() {
        let html = "<p>Hello</p>";
        let builder = TreeBuilder::new(html);
        let (dom, doc, _errors) = builder.parse();

        // Should have implicit html, head, body
        let html_elem = dom.get(doc).first_child;
        assert!(html_elem.is_some());
    }

    #[test]
    fn test_void_elements() {
        let html = "<p>Line 1<br>Line 2</p>";
        let builder = TreeBuilder::new(html);
        let (_dom, _doc, _errors) = builder.parse();
    }

    #[test]
    fn test_nested_elements() {
        let html = "<div><span><a href='#'>Link</a></span></div>";
        let builder = TreeBuilder::new(html);
        let (_dom, _doc, _errors) = builder.parse();
    }
}

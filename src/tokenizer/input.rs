//! Input stream for the tokenizer with position tracking.

/// An input stream for tokenizing HTML.
///
/// Provides character-by-character access with position tracking.
#[derive(Debug, Clone)]
pub struct InputStream<'a> {
    /// The remaining input.
    input: &'a str,
    /// Current byte offset in the original input.
    offset: usize,
    /// 1-indexed line number.
    line: u32,
    /// 1-indexed column number.
    column: u32,
    /// Positions of newlines for lookups.
    newlines: Vec<usize>,
    /// Length of the original input.
    original_len: usize,
}

impl<'a> InputStream<'a> {
    /// Create a new input stream from a string.
    pub fn new(input: &'a str) -> Self {
        // Pre-compute newline positions for efficient line/column lookups
        let newlines: Vec<usize> = input
            .bytes()
            .enumerate()
            .filter_map(|(i, b)| if b == b'\n' { Some(i) } else { None })
            .collect();

        Self {
            input,
            offset: 0,
            line: 1,
            column: 1,
            newlines,
            original_len: input.len(),
        }
    }

    /// Check if the input is exhausted.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.input.is_empty()
    }

    /// Get the current byte offset.
    #[inline]
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Get the current line number (1-indexed).
    #[inline]
    pub fn line(&self) -> u32 {
        self.line
    }

    /// Get the current column number (1-indexed).
    #[inline]
    pub fn column(&self) -> u32 {
        self.column
    }

    /// Get the current position as (line, column).
    #[inline]
    pub fn position(&self) -> (u32, u32) {
        (self.line, self.column)
    }

    /// Peek at the next character without consuming it.
    #[inline]
    pub fn peek(&self) -> Option<char> {
        self.input.chars().next()
    }

    /// Peek at the nth character (0-indexed) without consuming it.
    pub fn peek_n(&self, n: usize) -> Option<char> {
        self.input.chars().nth(n)
    }

    /// Peek at multiple characters and return them as a string slice.
    pub fn peek_str(&self, n: usize) -> &str {
        let end = self
            .input
            .char_indices()
            .nth(n)
            .map(|(i, _)| i)
            .unwrap_or(self.input.len());
        &self.input[..end]
    }

    /// Check if the input starts with a given string (case-insensitive).
    pub fn starts_with_ignore_case(&self, s: &str) -> bool {
        if self.input.len() < s.len() {
            return false;
        }
        self.input[..s.len()].eq_ignore_ascii_case(s)
    }

    /// Check if the input starts with a given string (case-sensitive).
    pub fn starts_with(&self, s: &str) -> bool {
        self.input.starts_with(s)
    }

    /// Consume and return the next character.
    pub fn next(&mut self) -> Option<char> {
        let mut chars = self.input.chars();
        let c = chars.next()?;

        // Update position
        let char_len = c.len_utf8();
        self.offset += char_len;
        self.input = chars.as_str();

        // Track line/column
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }

        Some(c)
    }

    /// Consume and return characters while a predicate holds.
    pub fn consume_while<F>(&mut self, mut predicate: F) -> &'a str
    where
        F: FnMut(char) -> bool,
    {
        let start = self.offset;
        let start_ptr = self.input.as_ptr();

        while let Some(c) = self.peek() {
            if !predicate(c) {
                break;
            }
            self.next();
        }

        let consumed_len = self.offset - start;
        // Safety: we're pointing into the original string, and these offsets are valid
        unsafe {
            let slice = std::slice::from_raw_parts(start_ptr, consumed_len);
            std::str::from_utf8_unchecked(slice)
        }
    }

    /// Skip n characters.
    pub fn skip(&mut self, n: usize) {
        for _ in 0..n {
            if self.next().is_none() {
                break;
            }
        }
    }

    /// Skip a specific string if it matches (case-insensitive).
    /// Returns true if the string was skipped.
    pub fn skip_str_ignore_case(&mut self, s: &str) -> bool {
        if self.starts_with_ignore_case(s) {
            self.skip(s.chars().count());
            true
        } else {
            false
        }
    }

    /// Get the remaining input as a string slice.
    pub fn remaining(&self) -> &str {
        self.input
    }

    /// Get the remaining length in bytes.
    pub fn remaining_len(&self) -> usize {
        self.input.len()
    }

    /// Compute line and column for a given offset.
    pub fn position_at(&self, offset: usize) -> (u32, u32) {
        if offset == 0 {
            return (1, 1);
        }

        // Binary search for the line containing this offset
        let line_idx = self.newlines.partition_point(|&pos| pos < offset);
        let line = (line_idx + 1) as u32;

        let line_start = if line_idx == 0 {
            0
        } else {
            self.newlines[line_idx - 1] + 1
        };

        let column = (offset - line_start + 1) as u32;
        (line, column)
    }

    /// Create a checkpoint that can be used to restore the stream state.
    pub fn checkpoint(&self) -> InputCheckpoint<'a> {
        InputCheckpoint {
            input: self.input,
            offset: self.offset,
            line: self.line,
            column: self.column,
        }
    }

    /// Restore the stream to a previous checkpoint.
    pub fn restore(&mut self, checkpoint: InputCheckpoint<'a>) {
        self.input = checkpoint.input;
        self.offset = checkpoint.offset;
        self.line = checkpoint.line;
        self.column = checkpoint.column;
    }
}

/// A checkpoint for restoring input stream state.
#[derive(Debug, Clone)]
pub struct InputCheckpoint<'a> {
    input: &'a str,
    offset: usize,
    line: u32,
    column: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_iteration() {
        let mut input = InputStream::new("abc");
        assert_eq!(input.next(), Some('a'));
        assert_eq!(input.next(), Some('b'));
        assert_eq!(input.next(), Some('c'));
        assert_eq!(input.next(), None);
        assert!(input.is_empty());
    }

    #[test]
    fn test_position_tracking() {
        let mut input = InputStream::new("ab\ncd\nef");
        assert_eq!(input.position(), (1, 1));

        input.next(); // a
        assert_eq!(input.position(), (1, 2));

        input.next(); // b
        assert_eq!(input.position(), (1, 3));

        input.next(); // \n
        assert_eq!(input.position(), (2, 1));

        input.next(); // c
        input.next(); // d
        input.next(); // \n
        assert_eq!(input.position(), (3, 1));
    }

    #[test]
    fn test_peek() {
        let input = InputStream::new("hello");
        assert_eq!(input.peek(), Some('h'));
        assert_eq!(input.peek_n(0), Some('h'));
        assert_eq!(input.peek_n(1), Some('e'));
        assert_eq!(input.peek_n(4), Some('o'));
        assert_eq!(input.peek_n(5), None);
    }

    #[test]
    fn test_peek_str() {
        let input = InputStream::new("hello world");
        assert_eq!(input.peek_str(5), "hello");
        assert_eq!(input.peek_str(11), "hello world");
        assert_eq!(input.peek_str(20), "hello world");
    }

    #[test]
    fn test_starts_with() {
        let input = InputStream::new("<!DOCTYPE html>");
        assert!(input.starts_with("<!"));
        assert!(input.starts_with_ignore_case("<!doctype"));
        assert!(input.starts_with_ignore_case("<!DOCTYPE"));
        assert!(!input.starts_with("<!doctype")); // case-sensitive
    }

    #[test]
    fn test_consume_while() {
        let mut input = InputStream::new("hello world");
        let consumed = input.consume_while(|c| c.is_alphabetic());
        assert_eq!(consumed, "hello");
        assert_eq!(input.peek(), Some(' '));
    }

    #[test]
    fn test_checkpoint_restore() {
        let mut input = InputStream::new("abcdef");
        input.next(); // a
        input.next(); // b

        let checkpoint = input.checkpoint();

        input.next(); // c
        input.next(); // d
        assert_eq!(input.peek(), Some('e'));

        input.restore(checkpoint);
        assert_eq!(input.peek(), Some('c'));
        assert_eq!(input.offset(), 2);
    }

    #[test]
    fn test_unicode() {
        let mut input = InputStream::new("héllo 世界");
        assert_eq!(input.next(), Some('h'));
        assert_eq!(input.next(), Some('é'));
        assert_eq!(input.next(), Some('l'));
        assert_eq!(input.next(), Some('l'));
        assert_eq!(input.next(), Some('o'));
        assert_eq!(input.next(), Some(' '));
        assert_eq!(input.next(), Some('世'));
        assert_eq!(input.next(), Some('界'));
        assert_eq!(input.next(), None);
    }
}

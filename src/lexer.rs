use crate::token::Token;

use std::iter::{Enumerate, Peekable};
use std::str::Bytes;

pub struct Lexer<'a> {
    src: &'a str,
    iter: Peekable<Enumerate<Bytes<'a>>>,
}

impl<'a> Lexer<'a> {
    /// Creates a new Lexer over GML source code.
    pub fn new(src: &'a str) -> Self {
        Lexer {
            src,
            iter: src.bytes().enumerate().peekable(),
        }
    }
    
    /// Fast-forwards the internal iterator to the next token, skipping over whitespace.
    fn fast_forward(&mut self) {
        // gml defines any ascii character that is ' ' and below as whitespace
        while self.iter.peek().map(|(_, ch)| *ch <= b' ').unwrap_or(false) {
            self.iter.next();
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Token<'a>> {
        self.fast_forward(); // locate next token

        None
    }
}

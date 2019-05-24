use crate::token::Token;

use std::iter::{Enumerate, Peekable};
use std::str::Bytes;

pub struct Lexer<'a> {
    src: &'a str,
    iter: Peekable<Enumerate<Bytes<'a>>>,
}

impl<'a> Lexer<'a> { 
    pub fn new(src: &'a str) -> Self {
        Lexer {
            src,
            iter: src.bytes().enumerate().peekable(),
        }
    }

    fn fast_forward(&mut self) {
        while self.iter.peek().map(|(_, ch)| *ch < (b' ' + 1)).unwrap_or(false) {
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

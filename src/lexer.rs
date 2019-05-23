use crate::token::Token;

use std::iter::Peekable;
use std::str::Chars;

pub struct Lexer<'a> {
    src: &'a str,
    iter: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> { 
    pub fn new(src: &'a str) -> Self {
        Lexer {
            src,
            iter: src.chars().peekable(),
        }
    }

    fn fast_forward(&mut self) {
        while self.iter.peek().map(|ch| ch.is_whitespace()).unwrap_or(false) {
            self.iter.next();
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Token<'a>> {
        Some(Token::String(&self.src[0..10]))
    }
}

use crate::token::*;

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
       
        // this is fine since we operate on something that is a &str in a first place
        // we should of course never use a value not pulled from peek() as range indices
        let src = self.src; // since &mut self
        macro_rules! to_str {
            ($src: expr, $range: expr) => ({
                unsafe {
                    std::str::from_utf8_unchecked(
                        $src.as_bytes()
                            .get_unchecked($range)
                    )
                }
            })
        };
        
        let head = *self.iter.peek()?;
        println!("we at '{}'", &self.src[(head.0)..]);
        match head.1 {
            b'A'...b'Z' | b'a'... b'z' | b'_' => {
                let identifier = {
                    let mut last = head;
                    loop {
                        match self.iter.next() {
                            Some(tail) => match tail.1 {
                                b'A'...b'Z' | b'a'...b'z' | b'0'...b'9' | b'_' => last = tail,
                                _ => break to_str!(src, head.0..tail.0),
                            },
                            None => break to_str!(src, head.0..=last.0),
                        }
                    }
                };
                return Some(Token::Identifier(identifier));
            },

            b'0' ..= b'9' | b'.' => {
                // inhale real
            },

            b'"' | b'\'' => {
                // inhale string
            },

            b'$' => {
                // inhale hex literal
            },

            0x00 ..= b'~' => {
                // operator possibly
            },

            _ => panic!("Oh no! Corrupt character: {} ({:#X})", head.1, head.1),
        }
        None
    }
}

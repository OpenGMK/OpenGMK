use crate::token::*;

use std::iter::{Enumerate, Peekable};
use std::str::{self, Bytes};

pub struct Lexer<'a> {
    src: &'a str,
    buf: Vec<u8>,
    iter: Peekable<Enumerate<Bytes<'a>>>,
}

impl<'a> Lexer<'a> {
    /// Creates a new Lexer over GML source code.
    pub fn new(src: &'a str) -> Self {
        Lexer {
            src,
            buf: Vec::with_capacity(8),
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
                    str::from_utf8_unchecked(
                        $src.as_bytes()
                            .get_unchecked($range)
                    )
                }
            })
        };
        
        let head = *self.iter.peek()?;
        Some(match head.1 {
            b'A'...b'Z' | b'a'... b'z' | b'_' => {
                let identifier = {
                    let mut last = head;
                    loop {
                        match self.iter.peek() {
                            Some(&tail) => match tail.1 {
                                b'A'...b'Z' | b'a'...b'z' | b'0'...b'9' | b'_' => {
                                    last = tail;
                                    self.iter.next();
                                },
                                _ => break to_str!(src, head.0..tail.0),
                            },
                            None => break to_str!(src, head.0..=last.0),
                        }
                    }
                };

                match identifier {
                    // Keywords
                    "var" => Token::Keyword(Keyword::Var),
                    "if" => Token::Keyword(Keyword::If),
                    "else" => Token::Keyword(Keyword::Else),
                    "with" => Token::Keyword(Keyword::With),
                    "repeat" => Token::Keyword(Keyword::Repeat),
                    "do" => Token::Keyword(Keyword::Do),
                    "until" => Token::Keyword(Keyword::Until),
                    "while" => Token::Keyword(Keyword::While),
                    "for" => Token::Keyword(Keyword::For),
                    "switch" => Token::Keyword(Keyword::Switch),
                    "case" => Token::Keyword(Keyword::Case),
                    "default" => Token::Keyword(Keyword::Default),
                    "break" => Token::Keyword(Keyword::Break),
                    "continue" => Token::Keyword(Keyword::Continue),
                    "return" => Token::Keyword(Keyword::Return),
                    "exit" => Token::Keyword(Keyword::Exit),

                    // Operators
                    "mod" => Token::Operator(Operator::Modulo),
                    "div" => Token::Operator(Operator::IntDivide),
                    "and" => Token::Operator(Operator::And),
                    "or" => Token::Operator(Operator::Or),
                    "xor" => Token::Operator(Operator::Xor),
                    "not" => Token::Operator(Operator::Not),
                    "then" => Token::Separator(Separator::Then),
                    "begin" => Token::Separator(Separator::BraceLeft),
                    "end" => Token::Separator(Separator::BraceRight),

                    _ => Token::Identifier(identifier),
                }
            },

            b'0'...b'9' | b'.' => {
                self.buf.clear();
                let mut has_decimal = false;
                loop {
                    match self.iter.peek() {
                        Some(&(_, ch)) => match ch {
                            b'0'...b'9' => {
                                self.buf.push(ch);
                                self.iter.next();
                            },
                            b'.' => {
                                self.iter.next();
                                if !has_decimal {
                                    has_decimal = true;
                                    self.buf.push(ch);
                                }
                            },
                            _ => break,
                        },
                        None => break,
                    }
                }
                if &self.buf == b"." {
                    Token::Separator(Separator::Period)
                } else {
                    Token::Real(
                        unsafe { str::from_utf8_unchecked(&self.buf) }
                            .parse()
                            .unwrap_or(0.0)
                    )
                }
            },

            b'"' | b'\'' => {
                // inhale string
                self.iter.next();
                Token::Identifier("invalid")
            },

            b'$' => {
                // inhale hex literal
                self.iter.next();
                Token::Identifier("invalid")
            },

            0x00 ..= b'~' => {
                // operator possibly
                self.iter.next();
                Token::Identifier("invalid")
            },

            _ => {
                self.iter.next(); // skip (if possible)
                Token::InvalidChar(head.0, head.1)
            },
        })
    }
}

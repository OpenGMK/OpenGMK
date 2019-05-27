use crate::token::*;

use std::iter::{Enumerate, Peekable};
use std::ops::Range;
use std::str::{self, Bytes};
use std::u64;

pub struct Lexer<'a> {
    /// GML source code to return references to.
    src: &'a str,

    /// Internal buffer for parsing numbers.
    /// Required due to a quirk described below.
    buf: Vec<u8>,

    /// Iterator over the source code as raw bytes.
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
        fn to_str<'a>(src: &'a str, range: Range<usize>) -> &'a str {
            unsafe {
                str::from_utf8_unchecked(src.as_bytes().get_unchecked(range))
            }
        }
        
        let head = *self.iter.peek()?;
        Some(match head.1 {
            // identifier, keyword or alphanumeric operator
            b'A'...b'Z' | b'a'... b'z' | b'_' => {
                let identifier = {
                    loop {
                        match self.iter.peek() {
                            Some(&tail) => match tail.1 {
                                b'A'...b'Z' | b'a'...b'z' | b'0'...b'9' | b'_' => { self.iter.next(); },
                                _ => break to_str(src, head.0..tail.0),
                            },
                            None => break to_str(src, head.0..src.len()),
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

            // real literal or . operator
            // in a real literal, every dot after the first one is ignored
            // a number can't begin with `..` - for example, '..1' is read as:
            // - the Period separator
            // - real literal literal 0.1
            // we copy this to self.buf, and the only purpose of this buffer is to be compliant
            // with this absolutely asinine language design, otherwise it could be non allocating.
            // examples of valid real literals:
            // 5.5.5.... => 5.55
            // 6...2...9 => 6.29
            // .7....3.. => 0.73
            // 4.2...0.0 => 4.2
            b'0'...b'9' | b'.' => {
                // whether we hit a . yet - begin ignoring afterwards if it's a real literal
                let mut has_decimal = false;
                self.buf.clear();
                loop {
                    match self.iter.peek() {
                        Some(&(_, ch)) => match ch {
                            b'0'...b'9' => {
                                self.buf.push(ch);
                                self.iter.next();
                            },
                            b'.' => {
                                if !has_decimal {
                                    has_decimal = true;
                                    self.buf.push(ch);
                                    self.iter.next();
                                } else {
                                    // correct interpretation of token starting with ..
                                    if &self.buf != b"." {
                                        self.iter.next();
                                    } else {
                                        break;
                                    }
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
                        // only 0-9 and . can be in the buffer, check unneeded
                        unsafe { str::from_utf8_unchecked(&self.buf) }
                            .parse()
                            .unwrap_or(0.0)
                    )
                }
            },

            // string literal
            // note: unclosed string literals at eof are accepted, however each script ends in:
            // newline
            // space
            // space
            // so "asdf would be "asdf\n  "
            // we don't take care of this here, that's the script loader's job
            b'"' | b'\'' => {
                self.iter.next(); // skip over opening quote
                let quote = head.1; // opening quote mark char

                // new head after opening quote
                let head = match self.iter.peek() {
                    Some(&(i, _)) => i,
                    None => return Some(Token::String("")),
                };

                let string = loop {
                    match self.iter.next() {
                        Some((i, ch)) => if ch == quote {
                            break to_str(src, head..i)
                        }, 
                        None => break to_str(src, head..src.len()),
                    }
                };
                Token::String(string)
            },

            // hexadecimal real literal.
            // a single $ with no valid hexadecimal chars after it is equivalent to $0.
            b'$' => {
                self.iter.next(); // skip '$'

                // new head after '$'
                let head = match self.iter.peek() {
                    Some(&(i, _)) => i,
                    None => return Some(Token::Real(0.0)),
                };

                let hex = loop {
                    match self.iter.peek() {
                        Some(&(i, ch)) => match ch {
                            b'0'...b'9' | b'a'...b'f' | b'A'...b'F' => { self.iter.next(); },
                            _ => break to_str(src, head..i),
                        },
                        None => break to_str(src, head..src.len()),
                    }
                };

                if hex.is_empty() {
                    Token::Real(0.0)
                } else {
                    Token::Real(
                        // if it failed to parse it must be too large, so we return the max value
                        u64::from_str_radix(hex, 16).unwrap_or(u64::MAX) as f64
                    )
                }
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

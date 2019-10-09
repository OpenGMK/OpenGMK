use super::token::{Keyword, Operator, Separator, Token};

use std::iter::{Enumerate, Peekable};
use std::ops::Range;
use std::str::{self, Bytes};
use std::u64;

pub struct Lexer<'a> {
    /// GML source code to return references to.
    src: &'a str,

    line_hint: usize,

    /// Iterator over the source code as raw bytes.
    iter: Peekable<Enumerate<Bytes<'a>>>,
}

impl<'a> Lexer<'a> {
    /// Creates a new Lexer over GML source code.
    pub fn new(src: &'a str) -> Self {
        Lexer {
            src,
            line_hint: 1,
            iter: src.bytes().enumerate().peekable(),
        }
    }

    /// Returns the current line number in the source code.
    pub fn line(&self) -> usize {
        self.line_hint
    }

    /// Fast-forwards the internal iterator to the next token, skipping over whitespace.
    /// Returns how many lines (LF) were skipped in the process.
    fn fast_forward(&mut self) -> usize {
        let mut lines_skipped: usize = 0;
        loop {
            match self.iter.peek() {
                Some(&(_, ch)) if ch <= b' ' => {
                    if ch == b'\n' {
                        lines_skipped += 1;
                    }
                    self.iter.next();
                }
                _ => break lines_skipped,
            }
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Token<'a>> {
        // locate next token
        self.line_hint = self.fast_forward();

        // this is fine since we operate on something that is a &str in a first place
        // we should of course never use a value not pulled from peek() as range indices
        let src = self.src; // since &mut self
        fn to_str<'a>(src: &'a str, range: Range<usize>) -> &'a str {
            unsafe { str::from_utf8_unchecked(src.as_bytes().get_unchecked(range)) }
        }

        let head = *self.iter.peek()?;
        Some(match head.1 {
            // identifier, keyword or alphanumeric operator/separator
            b'A'..=b'Z' | b'a'..=b'z' | b'_' => {
                let identifier = {
                    loop {
                        match self.iter.peek() {
                            Some(&tail) => match tail.1 {
                                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' => {
                                    self.iter.next();
                                }
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
            }

            // real literal or . operator
            // in a real literal, every dot after the first one is ignored
            // a number can't begin with `..` - for example, '..1' is read as:
            // - the Period separator
            // - real literal literal 0.1
            // examples of valid real literals, you will lose brain cells reading this:
            // 5.5.5.... => 5.55
            // 6...2...9 => 6.29
            // .7....3.. => 0.73
            // 4.2...0.0 => 4.2
            b'0'..=b'9' | b'.' => {
                let mut point_seen = false;

                match head.1 {
                    b'.' => {
                        self.iter.next();
                        match self.iter.peek() {
                            Some(&(_, ch)) => match ch {
                                b'0'..=b'9' => (),
                                _ => return Some(Token::Separator(Separator::Period)),
                            },
                            _ => (),
                        }
                    }
                    _ => (),
                }

                let mut result = 0.0f64;
                let mut factor = 1.0f64;
                loop {
                    match self.iter.peek() {
                        Some(&(_, ch)) => {
                            match ch {
                                ch @ b'0'..=b'9' => {
                                    let dec = ch - b'0';
                                    if point_seen {
                                        factor /= 10.0;
                                    }
                                    result = result * 10.0 + (dec as f64);
                                }
                                b'.' => point_seen = true,
                                _ => break,
                            }
                            self.iter.next();
                        }
                        None => break,
                    }
                }

                Token::Real(result * factor)
            }

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
                        Some((i, ch)) => {
                            if ch == quote {
                                break to_str(src, head..i);
                            }
                        }
                        None => break to_str(src, head..src.len()),
                    }
                };
                Token::String(string)
            }

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
                            b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F' => {
                                self.iter.next();
                            }
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
                        u64::from_str_radix(hex, 16).unwrap_or(u64::MAX) as f64,
                    )
                }
            }

            // operator, separator or possibly just an invalid character
            0x00..=b'~' => {
                let op_sep_ch = |ch| match ch & 0b0111_1111 {
                    b'!' => Token::Operator(Operator::Not),
                    b'&' => Token::Operator(Operator::BinaryAnd),
                    b'(' => Token::Separator(Separator::ParenLeft),
                    b')' => Token::Separator(Separator::ParenRight),
                    b'*' => Token::Operator(Operator::Multiply),
                    b'+' => Token::Operator(Operator::Add),
                    b',' => Token::Separator(Separator::Comma),
                    b'-' => Token::Operator(Operator::Subtract),
                    b'/' => Token::Operator(Operator::Divide),
                    b':' => Token::Separator(Separator::Colon),
                    b';' => Token::Separator(Separator::Semicolon),
                    b'<' => Token::Operator(Operator::LessThan),
                    b'=' => Token::Operator(Operator::Assign),
                    b'>' => Token::Operator(Operator::GreaterThan),
                    b'[' => Token::Separator(Separator::BracketLeft),
                    b']' => Token::Separator(Separator::BracketRight),
                    b'^' => Token::Operator(Operator::BinaryXor),
                    b'{' => Token::Separator(Separator::BraceLeft),
                    b'|' => Token::Operator(Operator::BinaryOr),
                    b'}' => Token::Separator(Separator::BraceRight),
                    b'~' => Token::Operator(Operator::Complement),
                    _ => Token::InvalidChar(head.0, head.1),
                };

                let token1 = op_sep_ch(head.1);
                self.iter.next();

                if let Token::Operator(op) = token1 {
                    let ch2 = match self.iter.peek() {
                        Some(&(_, ch)) => ch,
                        None => return Some(Token::Operator(op)),
                    };

                    // boolean operators that are just repeated chars
                    // such as && || ^^
                    if head.1 == ch2 {
                        let repeated_combo = match op {
                            Operator::BinaryAnd => Operator::And,
                            Operator::BinaryOr => Operator::Or,
                            Operator::BinaryXor => Operator::Xor,
                            Operator::LessThan => Operator::BinaryShiftLeft,
                            Operator::GreaterThan => Operator::BinaryShiftRight,

                            Operator::Assign => Operator::Equal,

                            // single line comments
                            Operator::Divide => {
                                self.iter.next();
                                loop {
                                    match self.iter.peek() {
                                        Some(&(_, ch)) => match ch {
                                            b'\n' | b'\r' => break,
                                            _ => {
                                                self.iter.next();
                                            }
                                        },
                                        None => break,
                                    }
                                }

                                return self.next();
                            }

                            _ => return Some(Token::Operator(op)),
                        };
                        self.iter.next();
                        Token::Operator(repeated_combo)
                    }
                    // assignment operator combos such as += -= *= /=
                    else if ch2 == b'=' {
                        let eq_combo = match op {
                            // boolean operators
                            // == is in above match condition since it's a repeated character
                            Operator::Not => Operator::NotEqual,

                            // comparison operators
                            Operator::LessThan => Operator::LessThanOrEqual,
                            Operator::GreaterThan => Operator::GreaterThanOrEqual,

                            // assignment operators
                            Operator::Add => Operator::AssignAdd,
                            Operator::Subtract => Operator::AssignSubtract,
                            Operator::Multiply => Operator::AssignMultiply,
                            Operator::Divide => Operator::AssignDivide,
                            Operator::BinaryAnd => Operator::AssignBinaryAnd,
                            Operator::BinaryOr => Operator::AssignBinaryOr,
                            Operator::BinaryXor => Operator::AssignBinaryXor,

                            _ => return Some(Token::Operator(op)),
                        };
                        self.iter.next();
                        Token::Operator(eq_combo)
                    }
                    // multi-line comments
                    else if op == Operator::Divide && ch2 == b'*' {
                        self.iter.next();
                        loop {
                            match self.iter.peek() {
                                Some(&(_, ch)) => match ch {
                                    b'*' => {
                                        self.iter.next();
                                        match self.iter.peek() {
                                            Some(&(_, ch)) => {
                                                if ch == b'/' {
                                                    self.iter.next();
                                                    break;
                                                }
                                            }
                                            None => break,
                                        }
                                    }
                                    _ => {
                                        self.iter.next();
                                    }
                                },
                                None => break,
                            }
                        }
                        return self.next();
                    } else {
                        Token::Operator(op)
                    }
                } else if let Token::Separator(Separator::Colon) = token1 {
                    // pascal-style := init-assignments
                    if self.iter.peek().map(|(_, ch)| *ch == b'=').unwrap_or(false) {
                        self.iter.next();
                        Token::Operator(Operator::Assign)
                    } else {
                        Token::Separator(Separator::Colon)
                    }
                } else {
                    token1
                }
            }

            // invalid unicode
            _ => {
                self.iter.next(); // skip (if possible)
                Token::InvalidChar(head.0, head.1)
            }
        })
    }
}

// The lexer is intrinsically tested via the AST tests.

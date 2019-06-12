#![allow(dead_code)]

mod lexer;
mod token;

use crate::lexer::Lexer;

fn main() {
    let gml = match std::env::args().skip(1).next() {
        Some(gml) => gml,
        None => {
            eprintln!("no input");
            return;
        },
    };

    let mut lexer = Lexer::new(&gml);
    while let Some(token) = lexer.next() {
        println!("{:?}", token);
    }
}


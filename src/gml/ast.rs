use super::lexer::Lexer;

pub struct AST {
    
}

impl AST {
    pub fn new(source: &str) -> Self {
        let lex = Lexer::new(source);
        let _tokens: Vec<_> = lex.collect();
        //println!("{:?}", tokens);
        AST{}
    }
}
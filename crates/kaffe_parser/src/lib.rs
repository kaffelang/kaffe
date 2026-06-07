mod lexer;
mod parser;

pub use lexer::{LexError, Span, Token, TokenWithSpan};
pub use parser::{ParseError, ParserError};

use kaffe_ast::Module;

pub fn parse(source: &str) -> Result<Module, ParserError> {
    let tokens = lexer::lex(source)?;
    parser::parse(tokens)
}

pub fn lex_tokens(source: &str) -> Result<Vec<Token>, LexError> {
    let tokens = lexer::lex(source)?;
    Ok(tokens.into_iter().map(|token| token.token).collect())
}

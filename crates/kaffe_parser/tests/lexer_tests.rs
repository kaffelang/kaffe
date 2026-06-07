use kaffe_parser::{lex_tokens, Token};

fn lex(src: &str) -> Vec<Token> {
    lex_tokens(src).unwrap()
}

#[test]
fn test_lex_keywords() {
    let tokens = lex("type fn if else in true false");
    assert_eq!(tokens[0], Token::KwType);
    assert_eq!(tokens[1], Token::KwFn);
    assert_eq!(tokens[2], Token::KwIf);
    assert_eq!(tokens[3], Token::KwElse);
    assert_eq!(tokens[4], Token::KwIn);
    assert_eq!(tokens[5], Token::KwTrue);
    assert_eq!(tokens[6], Token::KwFalse);
}

#[test]
fn test_lex_indentation() {
    let src = "type User =\n  id: string";
    let tokens = lex(src);
    assert!(tokens.contains(&Token::Indent));
    assert!(tokens.contains(&Token::Dedent));
}

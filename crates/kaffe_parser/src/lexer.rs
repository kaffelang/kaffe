use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    KwType,
    KwFn,
    KwIf,
    KwElse,
    KwIn,
    KwTrue,
    KwFalse,
    StringLit(String),
    NumberLit(f64),
    Ident(String),
    Arrow,
    EqEq,
    Neq,
    Assign,
    ExportMarker,
    Plus,
    Minus,
    Star,
    Slash,
    Gt,
    Lt,
    Gte,
    Lte,
    Pipe,
    Dot,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Colon,
    Question,
    Comma,
    Newline,
    Indent,
    Dedent,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenWithSpan {
    pub token: Token,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LexError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "lex error at {}:{}: {}", self.line, self.col, self.message)
    }
}

impl Error for LexError {}

pub fn lex(source: &str) -> Result<Vec<TokenWithSpan>, LexError> {
    let mut tokens = Vec::new();
    let mut indent_stack = vec![0usize];
    let mut first_real_line = true;

    for (line_idx, line) in source.lines().enumerate() {
        let line_no = line_idx + 1;
        let trimmed = line.trim();
        let trimmed_start = line.trim_start();

        if trimmed.is_empty() || trimmed_start.starts_with('#') {
            continue;
        }

        let indent = line.chars().take_while(|ch| *ch == ' ').count();
        let current_indent = *indent_stack.last().unwrap();

        if indent > current_indent {
            indent_stack.push(indent);
            tokens.push(TokenWithSpan {
                token: Token::Indent,
                span: Span { line: line_no, col: 1 },
            });
        } else if indent < current_indent {
            while indent < *indent_stack.last().unwrap() {
                indent_stack.pop();
                tokens.push(TokenWithSpan {
                    token: Token::Dedent,
                    span: Span { line: line_no, col: 1 },
                });
            }

            if indent != *indent_stack.last().unwrap() {
                return Err(LexError {
                    message: format!("invalid dedent to {} spaces", indent),
                    line: line_no,
                    col: 1,
                });
            }
        } else if !first_real_line {
            tokens.push(TokenWithSpan {
                token: Token::Newline,
                span: Span { line: line_no, col: 1 },
            });
        }

        first_real_line = false;
        lex_line(&line[indent..], line_no, indent + 1, &mut tokens)?;
    }

    while indent_stack.len() > 1 {
        indent_stack.pop();
        tokens.push(TokenWithSpan {
            token: Token::Dedent,
            span: Span { line: source.lines().count().max(1), col: 1 },
        });
    }

    tokens.push(TokenWithSpan {
        token: Token::Eof,
        span: Span {
            line: source.lines().count().max(1),
            col: source
                .lines()
                .last()
                .map(|line| line.chars().count() + 1)
                .unwrap_or(1),
        },
    });

    Ok(tokens)
}

fn lex_line(
    line: &str,
    line_no: usize,
    start_col: usize,
    tokens: &mut Vec<TokenWithSpan>,
) -> Result<(), LexError> {
    let chars: Vec<char> = line.chars().collect();
    let mut pos = 0usize;

    while pos < chars.len() {
        let ch = chars[pos];
        let col = start_col + pos;

        if ch == ' ' {
            pos += 1;
            continue;
        }

        if ch == '#' {
            break;
        }

        if is_ident_start(ch) {
            let start = pos;
            pos += 1;
            while pos < chars.len() && is_ident_continue(chars[pos]) {
                pos += 1;
            }
            let ident: String = chars[start..pos].iter().collect();
            let token = match ident.as_str() {
                "type" => Token::KwType,
                "fn" => Token::KwFn,
                "if" => Token::KwIf,
                "else" => Token::KwElse,
                "in" => Token::KwIn,
                "true" => Token::KwTrue,
                "false" => Token::KwFalse,
                _ => Token::Ident(ident),
            };
            tokens.push(TokenWithSpan {
                token,
                span: Span { line: line_no, col },
            });
            continue;
        }

        if ch.is_ascii_digit() {
            let start = pos;
            pos += 1;
            while pos < chars.len() && chars[pos].is_ascii_digit() {
                pos += 1;
            }
            if pos + 1 < chars.len() && chars[pos] == '.' && chars[pos + 1].is_ascii_digit() {
                pos += 1;
                while pos < chars.len() && chars[pos].is_ascii_digit() {
                    pos += 1;
                }
            }
            let number: String = chars[start..pos].iter().collect();
            let parsed = number.parse::<f64>().map_err(|_| LexError {
                message: format!("invalid number literal: {}", number),
                line: line_no,
                col,
            })?;
            tokens.push(TokenWithSpan {
                token: Token::NumberLit(parsed),
                span: Span { line: line_no, col },
            });
            continue;
        }

        if ch == '"' {
            let (value, consumed) = lex_string(&chars[pos..], line_no, col)?;
            tokens.push(TokenWithSpan {
                token: Token::StringLit(value),
                span: Span { line: line_no, col },
            });
            pos += consumed;
            continue;
        }

        let token = match ch {
            '=' => {
                if matches!(chars.get(pos + 1), Some('>')) {
                    pos += 2;
                    Token::Arrow
                } else if matches!(chars.get(pos + 1), Some('=')) {
                    pos += 2;
                    Token::EqEq
                } else {
                    pos += 1;
                    Token::Assign
                }
            }
            '!' => {
                if matches!(chars.get(pos + 1), Some('=')) {
                    pos += 2;
                    Token::Neq
                } else {
                    return Err(LexError {
                        message: "unexpected character '!'".to_string(),
                        line: line_no,
                        col,
                    });
                }
            }
            '>' => {
                if matches!(chars.get(pos + 1), Some('=')) {
                    pos += 2;
                    Token::Gte
                } else {
                    pos += 1;
                    Token::Gt
                }
            }
            '<' => {
                if matches!(chars.get(pos + 1), Some('=')) {
                    pos += 2;
                    Token::Lte
                } else {
                    pos += 1;
                    Token::Lt
                }
            }
            '+' => {
                // Check if this is an export marker (at start of statement) or arithmetic plus
                // Export marker appears when previous token is Newline, Indent, Dedent, or we're at start
                let is_export_marker = tokens.is_empty() 
                    || matches!(tokens.last().map(|t| &t.token), Some(Token::Newline) | Some(Token::Indent) | Some(Token::Dedent));
                pos += 1;
                if is_export_marker {
                    Token::ExportMarker
                } else {
                    Token::Plus
                }
            }
            '-' => {
                pos += 1;
                Token::Minus
            }
            '*' => {
                pos += 1;
                Token::Star
            }
            '/' => {
                pos += 1;
                Token::Slash
            }
            '|' => {
                pos += 1;
                Token::Pipe
            }
            '.' => {
                pos += 1;
                Token::Dot
            }
            '(' => {
                pos += 1;
                Token::LParen
            }
            ')' => {
                pos += 1;
                Token::RParen
            }
            '[' => {
                pos += 1;
                Token::LBracket
            }
            ']' => {
                pos += 1;
                Token::RBracket
            }
            ':' => {
                pos += 1;
                Token::Colon
            }
            '?' => {
                pos += 1;
                Token::Question
            }
            ',' => {
                pos += 1;
                Token::Comma
            }
            _ => {
                return Err(LexError {
                    message: format!("unexpected character '{}'", ch),
                    line: line_no,
                    col,
                })
            }
        };

        tokens.push(TokenWithSpan {
            token,
            span: Span { line: line_no, col },
        });
    }

    Ok(())
}

fn lex_string(chars: &[char], line_no: usize, col: usize) -> Result<(String, usize), LexError> {
    let mut out = String::new();
    let mut pos = 1usize;

    while pos < chars.len() {
        match chars[pos] {
            '"' => return Ok((out, pos + 1)),
            '\\' => {
                pos += 1;
                if pos >= chars.len() {
                    return Err(LexError {
                        message: "unterminated string literal".to_string(),
                        line: line_no,
                        col,
                    });
                }
                match chars[pos] {
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    other => out.push(other),
                }
                pos += 1;
            }
            other => {
                out.push(other);
                pos += 1;
            }
        }
    }

    Err(LexError {
        message: "unterminated string literal".to_string(),
        line: line_no,
        col,
    })
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_ident_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

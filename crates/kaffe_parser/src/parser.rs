use std::error::Error;
use std::fmt;

use crate::lexer::{LexError, Span, Token, TokenWithSpan};
use kaffe_ast::*;

#[derive(Debug)]
pub enum ParserError {
    Lex(LexError),
    Parse {
        message: String,
        line: usize,
        col: usize,
    },
}

pub type ParseError = ParserError;

struct Parser {
    tokens: Vec<TokenWithSpan>,
    pos: usize,
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParserError::Lex(e) => write!(f, "lex error at {}:{}: {}", e.line, e.col, e.message),
            ParserError::Parse { message, line, col } => {
                write!(f, "parse error at {}:{}: {}", line, col, message)
            }
        }
    }
}

impl Error for ParserError {}

impl From<LexError> for ParserError {
    fn from(value: LexError) -> Self {
        ParserError::Lex(value)
    }
}

pub fn parse(tokens: Vec<TokenWithSpan>) -> Result<Module, ParserError> {
    Parser { tokens, pos: 0 }.parse_module()
}

impl Parser {
    fn parse_module(&mut self) -> Result<Module, ParserError> {
        let mut items = Vec::new();

        loop {
            while self.peek() == &Token::Newline {
                self.advance();
            }

            if self.peek() == &Token::Eof {
                break;
            }

            items.push(self.parse_item()?);
        }

        Ok(Module { items })
    }

    fn parse_item(&mut self) -> Result<Item, ParserError> {
        match self.peek() {
            Token::ExportMarker => {
                self.advance();
                let item = self.parse_item()?;
                Ok(Item::Export(Box::new(item)))
            }
            Token::KwType => Ok(Item::TypeAlias(self.parse_type_alias()?)),
            Token::KwFn => Ok(Item::Function(self.parse_function()?)),
            _ => Err(self.error_here("expected item")),
        }
    }

    fn parse_type_alias(&mut self) -> Result<TypeAlias, ParserError> {
        self.expect_kind(&Token::KwType)?;
        let name = self.expect_ident()?;
        self.expect_kind(&Token::Assign)?;
        self.expect_kind(&Token::Indent)?;

        let mut fields = Vec::new();
        loop {
            fields.push(self.parse_type_field()?);
            match self.peek() {
                Token::Newline => {
                    self.advance();
                }
                Token::Dedent => {
                    self.advance();
                    break;
                }
                _ => return Err(self.error_here("expected newline or dedent after type field")),
            }
        }

        Ok(TypeAlias { name, fields })
    }

    fn parse_type_field(&mut self) -> Result<TypeField, ParserError> {
        let name = self.expect_ident()?;
        let mut optional = false;
        if self.peek() == &Token::Question {
            self.advance();
            optional = true;
        }
        self.expect_kind(&Token::Colon)?;
        let ty = self.parse_type_expr()?;
        Ok(TypeField { name, optional, ty })
    }

    fn parse_type_expr(&mut self) -> Result<TypeExpr, ParserError> {
        let mut exprs = vec![self.parse_type_atom()?];
        while self.peek() == &Token::Pipe {
            self.advance();
            exprs.push(self.parse_type_atom()?);
        }
        if exprs.len() == 1 {
            Ok(exprs.remove(0))
        } else {
            Ok(TypeExpr::Union(exprs))
        }
    }

    fn parse_type_atom(&mut self) -> Result<TypeExpr, ParserError> {
        let mut ty = match self.advance() {
            Token::StringLit(value) => TypeExpr::StringLit(value),
            Token::Ident(value) => TypeExpr::Named(value),
            other => {
                return Err(self.error_at_current(format!("expected type expression, found {:?}", other)))
            }
        };

        while self.peek() == &Token::LBracket && self.peek_next() == Some(&Token::RBracket) {
            self.advance();
            self.advance();
            ty = TypeExpr::Array(Box::new(ty));
        }

        Ok(ty)
    }

    fn parse_function(&mut self) -> Result<FunctionDecl, ParserError> {
        self.expect_kind(&Token::KwFn)?;
        let name = self.expect_ident()?;
        let params = self.parse_params()?;
        let return_type = if self.peek() == &Token::Colon {
            self.advance();
            Some(self.parse_type_expr()?)
        } else {
            None
        };
        self.expect_kind(&Token::Arrow)?;
        let body = self.parse_indented_expr()?;
        Ok(FunctionDecl {
            name,
            params,
            return_type,
            body,
        })
    }

    fn parse_params(&mut self) -> Result<Vec<Param>, ParserError> {
        self.expect_kind(&Token::LParen)?;
        let mut params = Vec::new();

        if self.peek() == &Token::RParen {
            self.advance();
            return Ok(params);
        }

        loop {
            let name = self.expect_ident()?;
            self.expect_kind(&Token::Colon)?;
            let ty = self.parse_type_expr()?;
            params.push(Param { name, ty });

            if self.peek() == &Token::Comma {
                self.advance();
            } else {
                break;
            }
        }

        self.expect_kind(&Token::RParen)?;
        Ok(params)
    }

    fn parse_expr(&mut self) -> Result<Expr, ParserError> {
        self.parse_in_expr()
    }

    fn parse_in_expr(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.parse_comparison()?;
        while self.peek() == &Token::KwIn {
            self.advance();
            let right = self.parse_comparison()?;
            expr = Expr::In(Box::new(expr), Box::new(right));
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.parse_additive()?;

        loop {
            let op = match self.peek() {
                Token::EqEq => BinOp::Eq,
                Token::Neq => BinOp::Neq,
                Token::Gt => BinOp::Gt,
                Token::Lt => BinOp::Lt,
                Token::Gte => BinOp::Gte,
                Token::Lte => BinOp::Lte,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive()?;
            expr = Expr::BinaryOp(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn parse_additive(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.parse_multiplicative()?;

        loop {
            let op = match self.peek() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            expr = Expr::BinaryOp(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.parse_member_access()?;

        loop {
            let op = match self.peek() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                _ => break,
            };
            self.advance();
            let right = self.parse_member_access()?;
            expr = Expr::BinaryOp(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn parse_member_access(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.parse_primary()?;
        while self.peek() == &Token::Dot {
            self.advance();
            let field = self.expect_ident()?;
            expr = Expr::MemberAccess(Box::new(expr), field);
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParserError> {
        match self.peek().clone() {
            Token::KwIf => self.parse_if_expr(),
            Token::KwTrue => {
                self.advance();
                Ok(Expr::BoolLiteral(true))
            }
            Token::KwFalse => {
                self.advance();
                Ok(Expr::BoolLiteral(false))
            }
            Token::StringLit(value) => {
                let span = self.peek_span().clone();
                self.advance();
                Self::parse_string_or_template(value, &span)
            }
            Token::NumberLit(value) => {
                self.advance();
                Ok(Expr::NumberLiteral(value))
            }
            Token::Ident(value) => {
                self.advance();
                Ok(Expr::Identifier(value))
            }
            Token::LBracket => self.parse_array_expr(),
            Token::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect_kind(&Token::RParen)?;
                Ok(expr)
            }
            other => Err(self.error_at_current(format!("expected expression, found {:?}", other))),
        }
    }

    fn parse_array_expr(&mut self) -> Result<Expr, ParserError> {
        self.expect_kind(&Token::LBracket)?;
        let mut elems = Vec::new();

        if self.peek() == &Token::RBracket {
            self.advance();
            return Ok(Expr::Array(elems));
        }

        loop {
            elems.push(self.parse_expr()?);
            if self.peek() == &Token::Comma {
                self.advance();
            } else {
                break;
            }
        }

        self.expect_kind(&Token::RBracket)?;
        Ok(Expr::Array(elems))
    }

    fn parse_if_expr(&mut self) -> Result<Expr, ParserError> {
        self.expect_kind(&Token::KwIf)?;
        let condition = self.parse_expr()?;
        let then_branch = self.parse_indented_expr()?;
        let else_branch = if self.peek() == &Token::KwElse {
            self.advance();
            Some(Box::new(self.parse_indented_expr()?))
        } else {
            None
        };

        Ok(Expr::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    fn parse_indented_expr(&mut self) -> Result<Expr, ParserError> {
        self.expect_kind(&Token::Indent)?;
        let expr = self.parse_expr()?;
        self.expect_kind(&Token::Dedent)?;
        Ok(expr)
    }

    fn parse_string_or_template(s: String, span: &Span) -> Result<Expr, ParserError> {
        if !s.contains("#{") {
            return Ok(Expr::StringLiteral(s));
        }

        let mut parts = Vec::new();
        let mut remaining = s.as_str();

        while let Some(start) = remaining.find("#{") {
            if start > 0 {
                parts.push(TemplatePart::Literal(remaining[..start].to_string()));
            }

            let after = &remaining[start + 2..];
            if let Some(end) = after.find('}') {
                let expr_str = &after[..end];
                let interpolation_col = span.col + remaining[..start].chars().count() + 1;
                let expr = Self::parse_dotted_ident(expr_str, span.line, interpolation_col)?;
                parts.push(TemplatePart::Interpolated(Box::new(expr)));
                remaining = &after[end + 1..];
            } else {
                let interpolation_col = span.col + remaining[..start].chars().count() + 1;
                return Err(ParserError::Parse {
                    message: "unterminated template interpolation".to_string(),
                    line: span.line,
                    col: interpolation_col,
                });
            }
        }

        if !remaining.is_empty() {
            parts.push(TemplatePart::Literal(remaining.to_string()));
        }

        Ok(Expr::TemplateLiteral(parts))
    }

    fn parse_dotted_ident(s: &str, line: usize, col: usize) -> Result<Expr, ParserError> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(ParserError::Parse {
                message: "empty template interpolation".to_string(),
                line,
                col,
            });
        }

        let mut parts = trimmed.split('.');
        let first = parts.next().expect("trimmed string is not empty");
        if !is_valid_ident(first) {
            return Err(ParserError::Parse {
                message: format!("invalid template interpolation: {}", trimmed),
                line,
                col,
            });
        }

        let mut expr = Expr::Identifier(first.to_string());
        for part in parts {
            if !is_valid_ident(part) {
                return Err(ParserError::Parse {
                    message: format!("invalid template interpolation: {}", trimmed),
                    line,
                    col,
                });
            }
            expr = Expr::MemberAccess(Box::new(expr), part.to_string());
        }
        Ok(expr)
    }

    fn peek(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .map(|token| &token.token)
            .unwrap_or(&self.tokens.last().expect("parser requires EOF token").token)
    }

    fn peek_next(&self) -> Option<&Token> {
        self.tokens.get(self.pos + 1).map(|token| &token.token)
    }

    fn peek_span(&self) -> &Span {
        self.tokens
            .get(self.pos)
            .map(|token| &token.span)
            .unwrap_or(&self.tokens.last().expect("parser requires EOF token").span)
    }

    fn advance(&mut self) -> Token {
        let token = self.peek().clone();
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        token
    }

    fn expect_kind(&mut self, expected: &Token) -> Result<(), ParserError> {
        if same_kind(self.peek(), expected) {
            self.advance();
            Ok(())
        } else {
            Err(self.error_at_current(format!("expected {:?}, found {:?}", expected, self.peek())))
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParserError> {
        match self.advance() {
            Token::Ident(value) => Ok(value),
            other => Err(self.error_at_current(format!("expected identifier, found {:?}", other))),
        }
    }

    fn error_here(&self, message: impl Into<String>) -> ParserError {
        let span = self.peek_span();
        ParserError::Parse {
            message: message.into(),
            line: span.line,
            col: span.col,
        }
    }

    fn error_at_current(&self, message: impl Into<String>) -> ParserError {
        self.error_here(message)
    }
}

fn same_kind(left: &Token, right: &Token) -> bool {
    std::mem::discriminant(left) == std::mem::discriminant(right)
}

fn is_valid_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(ch) if ch.is_ascii_alphabetic() || ch == '_' => {}
        _ => return false,
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

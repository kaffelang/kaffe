use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Item {
    TypeAlias(TypeAlias),
    Function(FunctionDecl),
    Export(Box<Item>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeAlias {
    pub name: String,
    pub fields: Vec<TypeField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeField {
    pub name: String,
    pub optional: bool,
    pub ty: TypeExpr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeExpr {
    Named(String),
    StringLit(String),
    Union(Vec<TypeExpr>),
    Array(Box<TypeExpr>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<TypeExpr>,
    pub body: Expr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    pub ty: TypeExpr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expr {
    StringLiteral(String),
    TemplateLiteral(Vec<TemplatePart>),
    NumberLiteral(f64),
    BoolLiteral(bool),
    Identifier(String),
    MemberAccess(Box<Expr>, String),
    BinaryOp(Box<Expr>, BinOp, Box<Expr>),
    In(Box<Expr>, Box<Expr>),
    Array(Vec<Expr>),
    If {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Option<Box<Expr>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemplatePart {
    Literal(String),
    Interpolated(Box<Expr>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
    Gt,
    Lt,
    Gte,
    Lte,
}

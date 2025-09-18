#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Expr(Expr),
    FuncDef {
        ident: Ident,
        args: Vec<Ident>,
        expr: Expr,
    },
    Let {
        ident: Ident,
        value: Expr,
    },
    For {
        ident: Ident,
        iter: Expr,
        expr: Expr,
    },
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Unary {
        operand: Box<Expr>,
        operator: UnaryOperator,
    },
    Binary {
        lhs: Box<Expr>,
        operator: BinaryOperator,
        rhs: Box<Expr>,
    },
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Minus,
    Call(Vec<Expr>),
    Index(Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Assign,
    Equal,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Number(i32),
    Bool(bool),
    String(String),
    Array(Vec<Expr>),
    Ident(Ident),
    If {
        condition: Box<Expr>,
        expr: Box<Expr>,
        or: Option<Box<Expr>>,
    },
    Block {
        statements: Vec<Statement>,
        return_value: Box<Expr>,
    },
    None,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Ident(pub String);

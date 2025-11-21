// src/ast.rs
pub type ElseIf = (Expr, Vec<Stmt>);

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Ident(String),
    List(Vec<Expr>),
    Binary { 
        left: Box<Expr>, 
        op: crate::lexer::Token, 
        right: Box<Expr> 
    },
    Unary { 
        op: crate::lexer::Token, 
        expr: Box<Expr> 
    },
    Call { 
        callee: String, 
        args: Vec<Expr> 
    },
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
        value: Option<Box<Expr>>,
    },
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Const { 
        name: String, 
        value: Expr 
    },
    Mutable { 
        name: String, 
        init: Option<Expr> 
    },
    Assign { 
        name: String, 
        value: Expr 
    },
    Expr(Expr),
    Return(Option<Expr>),
    Course {           // Procedures (no return value)
        name: String, 
        params: Vec<String>, 
        body: Vec<Stmt> 
    },
    Purpose {          // Functions (can return values with yield)
        name: String, 
        params: Vec<String>, 
        body: Vec<Stmt> 
    },
    Persist { 
        cond: Expr, 
        body: Vec<Stmt> 
    },
    When { 
        cond: Expr, 
        then: Vec<Stmt>, 
        elifs: Vec<ElseIf>, 
        otherwise: Vec<Stmt> 
    },
    Iterate { 
        var: String, 
        iterable: Expr, 
        body: Vec<Stmt> 
    },
    Block(Vec<Stmt>),
}
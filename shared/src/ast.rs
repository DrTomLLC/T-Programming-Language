//! Central AST definitions for T‑Lang: operators, expressions, and statements.

/// Binary operators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    EqualEqual,
    NotEqual,
    And,
    Or,
}

/// Unary operators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    Negate,
    Not,
}

/// Expression AST node.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    LiteralNumber(f64),
    LiteralBool(bool),
    LiteralString(String),
    Variable(String),
    Grouping(Box<Expr>),
    Unary { op: UnaryOp, expr: Box<Expr> },
    Binary { left: Box<Expr>, op: BinaryOp, right: Box<Expr> },
    Call(String, Vec<Expr>),
    ListLiteral(Vec<Expr>),
    If {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
    Block(Vec<Stmt>),
}

/// Statement AST node.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr(Expr),
    Let(String, Expr),
    Assign(String, Expr),
    If {
        cond: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Vec<Stmt>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    Block(Vec<Stmt>),
    Function(String, Vec<String>, Vec<Stmt>),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ast_variants_smoke() {
        // Binary and unary operators
        let _ = BinaryOp::Add;
        let _ = UnaryOp::Not;

        // Expression variants
        let n = Expr::LiteralNumber(0.0);
        assert!(matches!(n, Expr::LiteralNumber(_)));
        let b = Expr::LiteralBool(true);
        assert!(matches!(b, Expr::LiteralBool(_)));
        let s = Expr::LiteralString("hi".into());
        assert!(matches!(s, Expr::LiteralString(_)));
        let v = Expr::Variable("x".into());
        assert!(matches!(v, Expr::Variable(_)));
        let g = Expr::Grouping(Box::new(Expr::LiteralNumber(1.0)));
        assert!(matches!(g, Expr::Grouping(_)));
        let u = Expr::Unary { op: UnaryOp::Negate, expr: Box::new(Expr::LiteralNumber(1.0)) };
        assert!(matches!(u, Expr::Unary { .. }));
        let c = Expr::Call("f".into(), vec![]);
        assert!(matches!(c, Expr::Call(_, _)));
        let l = Expr::ListLiteral(vec![Expr::LiteralNumber(2.0)]);
        assert!(matches!(l, Expr::ListLiteral(_)));
        let i = Expr::If {
            condition: Box::new(Expr::LiteralBool(true)),
            then_branch: Box::new(Expr::LiteralNumber(1.0)),
            else_branch: Box::new(Expr::LiteralNumber(0.0)),
        };
        assert!(matches!(i, Expr::If { .. }));
        let bl = Expr::Block(vec![]);
        assert!(matches!(bl, Expr::Block(_)));

        // Statement variants
        let st = Stmt::Expr(Expr::LiteralBool(false));
        assert!(matches!(st, Stmt::Expr(_)));
        let st2 = Stmt::Let("y".into(), Expr::LiteralNumber(3.0));
        assert!(matches!(st2, Stmt::Let(_, _)));
        let st3 = Stmt::Assign("z".into(), Expr::LiteralNumber(4.0));
        assert!(matches!(st3, Stmt::Assign(_, _)));
        let st4 = Stmt::Block(vec![]);
        assert!(matches!(st4, Stmt::Block(_)));
    }
}
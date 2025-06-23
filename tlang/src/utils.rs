// tlang/src/utils.rs

use shared::ast::Stmt;

/// Helper to get a human-readable name for each AST variant.
pub fn stmt_name(s: &Stmt) -> &'static str {
    match s {
        Stmt::Expr(_)           => "Expr",
        Stmt::Let(_, _)         => "Let",
        Stmt::Assign(_, _)      => "Assign",
        Stmt::If { .. }         => "If",
        Stmt::While { .. }      => "While",
        Stmt::Block(_)          => "Block",
        Stmt::Function(_, _, _) => "Function",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::ast::{Expr, Stmt};

    #[test]
    fn test_stmt_name_expr() {
        let stmt = Stmt::Expr(Expr::LiteralBool(true));
        assert_eq!(stmt_name(&stmt), "Expr");
    }

    #[test]
    fn test_stmt_name_let() {
        let expr = Expr::LiteralNumber(42.0);
        let stmt = Stmt::Let("x".into(), expr);
        assert_eq!(stmt_name(&stmt), "Let");
    }
}

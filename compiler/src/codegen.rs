//! Lower the AST into a simple IR (`Value`) along with source‐spans.

use miette::SourceSpan;
use errors::{TlError, ErrorCode};
use shared::ast::{Expr, ExprKind, Literal, Pattern, Span as AstSpan, Stmt};

/// Our IR: a flat sequence of instructions/values.
#[derive(Debug, Clone)]
pub enum Value {
    LiteralNumber(f64, SourceSpan),
    LiteralBool(bool, SourceSpan),
    LiteralString(String, SourceSpan),
    GetVar(String, SourceSpan),
    SetVar(String, Box<Value>, SourceSpan),
    Block(Vec<Value>, SourceSpan),
    Number(f64, SourceSpan),   // alias if you prefer the shorter name
    Bool(bool, SourceSpan),
    String(String, SourceSpan),
    List(Vec<Value>, SourceSpan),
    Add(Box<Value>, Box<Value>, SourceSpan),
    Sub(Box<Value>, Box<Value>, SourceSpan),
    Mul(Box<Value>, Box<Value>, SourceSpan),
    Div(Box<Value>, Box<Value>, SourceSpan),
    Less(Box<Value>, Box<Value>, SourceSpan),
    Greater(Box<Value>, Box<Value>, SourceSpan),
    LessEqual(Box<Value>, Box<Value>, SourceSpan),
    GreaterEqual(Box<Value>, Box<Value>, SourceSpan),
    EqualEqual(Box<Value>, Box<Value>, SourceSpan),
    NotEqual(Box<Value>, Box<Value>, SourceSpan),
    And(Box<Value>, Box<Value>, SourceSpan),
    Or(Box<Value>, Box<Value>, SourceSpan),
    Not(Box<Value>, SourceSpan),
    Function { name: String, params: Vec<String>, body: Box<Value>, span: SourceSpan },
    Call(String, Vec<Value>, SourceSpan),
    If { cond: Box<Value>, then_branch: Box<Value>, else_branch: Option<Box<Value>>, span: SourceSpan },
    Return(Box<Value>, SourceSpan),
    /// …and so on for each operator your eval() matches…
    /// Preserve explicit grouping parentheses
    Group(Box<Value>, SourceSpan),
    // … you can add more IR ops here (binary, call, etc.) …
}
impl Value {
    pub fn span(&self) -> SourceSpan {
        match self {
            Value::Number(_, s)
            | Value::Bool(_, s)
            | Value::String(_, s)
            | Value::List(_, s)
            | Value::Add(_, _, s)
            // …repeat for every variant that carries a `SourceSpan`…
            | Value::Block(_, s)
            | Value::Group(_, s)
            | Value::GetVar(_, s)
            | Value::SetVar(_, _, s)
            => *s,
            // Should never happen as all variants contain a SourceSpan
            _ => SourceSpan::from((0, 0))
        }
    }
}

pub fn generate(stmts: &[Stmt]) -> Result<Vec<Value>, TlError> {
    let mut result = Vec::new();
    for stmt in stmts {
        match stmt {
            Stmt::Local { pat, init, span, .. } => {
                if let Some(init_expr) = init {
                    let v = gen_expr(init_expr)?;
                    result.push(Value::SetVar(
                        extract_ident(pat)?,
                        Box::new(v),
                        span_to_sourcespan(*span),
                    ));
                }
            }
            Stmt::Expr { expr, .. } | Stmt::Semi { expr, .. } => {
                let v = gen_expr(expr)?;
                result.push(v);
            }
            _ => {}
        }
    }
    Ok(result)
}

fn gen_expr(expr: &Expr) -> Result<Value, TlError> {
    match &expr.kind {
        ExprKind::Literal(Literal::Float(n), span) => {
            Ok(Value::LiteralNumber(*n, span_to_sourcespan(*span)))
        }
        ExprKind::Literal(Literal::Boolean(b), span) => {
            Ok(Value::LiteralBool(*b, span_to_sourcespan(*span)))
        }
        ExprKind::Literal(Literal::String(s), span) => {
            Ok(Value::LiteralString(s.clone(), span_to_sourcespan(*span)))
        }

        ExprKind::Variable(path, span) => {
            let name = path.join("::");
            Ok(Value::GetVar(name, span_to_sourcespan(*span)))
        }

        ExprKind::Grouping(inner, span) => {
            // generate inner IR, then wrap in a Group to preserve the span
            let inner_ir = gen_expr(inner)?;
            Ok(Value::Group(Box::new(inner_ir), span_to_sourcespan(*span)))
        }

        ExprKind::BlockExpr(block) => {
            let inner = generate(&block.stmts)?;
            Ok(Value::Block(inner, span_to_sourcespan(block.span)))
        }

        other => Err(TlError::new(
            "codegen",
            "",
            span_to_sourcespan(expr_span(expr)),
            ErrorCode::Unsupported,
            format!("unhandled expr in codegen: {:?}", other),
        )),
    }
}

/// Convert our AST `Span` into a `SourceSpan`.
fn span_to_sourcespan(span: AstSpan) -> SourceSpan {
    SourceSpan::from((span.start, span.len))
}

/// Extract an identifier name out of a single‐binding pattern.
fn extract_ident(pat: &Pattern) -> Result<String, TlError> {
    if let Pattern::Identifier { name, .. } = pat {
        Ok(name.clone())
    } else {
        Err(TlError::new(
            "codegen",
            "",
            SourceSpan::from((0, 0)),
            ErrorCode::Unsupported,
            "expected identifier in codegen".to_string(),
        ))
    }
}

/// Give us easy access to an expr’s AST span.
fn expr_span(expr: &Expr) -> AstSpan {
    use ExprKind::*;
    match &expr.kind {
        Literal(_, span)
        | Variable(_, span)
        | Grouping(_, span) => *span,
        BlockExpr(block) => block.span,
        _ => AstSpan { start: 0, len: 0 }, // fallback for unhandled kinds
    }
}

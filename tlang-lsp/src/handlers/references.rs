// tlang-lsp/src/handlers/references.rs

use tower_lsp::lsp_types::*;
use tower_lsp::jsonrpc::Result;
use crate::handlers::diagnostics::Backend;
use shared::tokenizer::tokenize;
use compiler::parser::Parser;
use shared::ast::*;
use std::sync::Arc;

/// Handle “textDocument/references” requests.
pub async fn handle_references(
    backend: &Backend,
    params: ReferenceParams,
) -> Result<Option<Vec<Location>>> {
    let uri = params.text_document_position.text_document.uri.clone();
    let pos = params.text_document_position.position;

    // 1) Load document text
    let docs = backend.docs.read().await;
    let source = if let Some(src) = docs.get(&uri) {
        src.clone()
    } else {
        return Ok(None);
    };
    drop(docs);

    // 2) Tokenize + parse
    let tokens = tokenize(&source)
        .map_err(|_| tower_lsp::jsonrpc::Error::invalid_params("lex error"))?;
    let mut parser = Parser::from_tokens(tokens.clone())
        .map_err(|_| tower_lsp::jsonrpc::Error::invalid_params("parse error"))?;
    let items = parser.parse_module()
        .map_err(|_| tower_lsp::jsonrpc::Error::invalid_params("parse error"))?;

    // 3) Find identifier under cursor
    let byte_offset = {
        let mut line = 0_usize;
        let mut col = 0_usize;
        for (i, ch) in source.char_indices() {
            if (line as u32, col as u32) == (pos.line, pos.character) {
                break;
            }
            if ch == '\n' { line += 1; col = 0; }
            else { col += 1; }
        }
        byte_offset.min(source.len())
    };
    let name = tokens.iter()
        .find(|t| t.span.start <= byte_offset && byte_offset < t.span.start + t.span.len)
        .and_then(|t| {
            if t.kind == shared::token::TokenType::Identifier {
                Some(t.lexeme.clone())
            } else {
                None
            }
        });
    let name = if let Some(n) = name { n } else {
        return Ok(None);
    };

    // 4) Walk AST + body expressions to collect all spans where `name` appears
    let mut locations = Vec::new();
    collect_references(&items, &uri, &name, &mut locations);

    Ok(Some(locations))
}

/// Recursively search AST and record any reference to `name`.
fn collect_references(
    items: &[Item],
    uri: &Url,
    name: &str,
    out: &mut Vec<Location>,
) {
    use Item::*;
    for item in items {
        match item {
            Function { signature, body, span, .. } => {
                if signature.name == *name {
                    out.push(location_for(span, uri));
                }
                for stmt in &body.stmts {
                    collect_stmt_refs(stmt, uri, name, out);
                }
            }
            Const { name: c, span, .. } if c == name => {
                out.push(location_for(span, uri));
            }
            Static { name: s, span, .. } if s == name => {
                out.push(location_for(span, uri));
            }
            Struct { name: s, span, .. } if s == name => {
                out.push(location_for(span, uri));
            }
            Enum { name: e, span, .. } if e == name => {
                out.push(location_for(span, uri));
            }
            Use { path, span, .. } if path.last().map(|p| p == name).unwrap_or(false) => {
                out.push(location_for(span, uri));
            }
            Module { items: inner, .. }
            | Trait { items: inner, .. }
            | Impl { items: inner, .. }
            | ExternBlock { items: inner, .. } => {
                collect_references(inner, uri, name, out);
            }
            _ => {},
        }
    }
}

fn collect_stmt_refs(
    stmt: &Stmt,
    uri: &Url,
    name: &str,
    out: &mut Vec<Location>,
) {
    match stmt {
        Stmt::Local { pat, init, span, .. } => {
            if pat_contains(pat, name) {
                out.push(location_for(span, uri));
            }
            if let Some(expr) = init {
                collect_expr_refs(expr, uri, name, out);
            }
        }
        Stmt::Expr { expr, span } | Stmt::Semi { expr, span } => {
            if expr_contains(expr, name) {
                out.push(location_for(span, uri));
            }
        }
        Stmt::Item(item) => collect_references(&[*item.clone()], uri, name, out),
        _ => {}
    }
}

fn collect_expr_refs(
    expr: &Expr,
    uri: &Url,
    name: &str,
    out: &mut Vec<Location>,
) {
    match expr {
        Expr::Variable(path, span) => {
            if path.last().map(|s| s == name).unwrap_or(false) {
                out.push(location_for(span, uri));
            }
        }
        Expr::Binary { left, right, span, .. } => {
            collect_expr_refs(left, uri, name, out);
            collect_expr_refs(right, uri, name, out);
            // binary itself not named
        }
        Expr::Call { func, args, span } => {
            collect_expr_refs(func, uri, name, out);
            for arg in args { collect_expr_refs(arg, uri, name, out); }
        }
        Expr::MethodCall { receiver, args, span, .. } => {
            collect_expr_refs(receiver, uri, name, out);
            for arg in args { collect_expr_refs(arg, uri, name, out); }
        }
        Expr::If { cond, then_branch, else_branch, span } => {
            collect_expr_refs(cond, uri, name, out);
            for stmt in &then_branch.stmts { collect_stmt_refs(stmt, uri, name, out); }
            if let Some(eb) = else_branch {
                collect_expr_refs(eb, uri, name, out);
            }
        }
        // ... handle other variants similarly, recursing into nested Expr and Stmt ...
        _ => {}
    }
}

fn pat_contains(pat: &Pattern, name: &str) -> bool {
    match pat {
        Pattern::Identifier(id) => id == name,
        Pattern::Reference { pat, .. } => pat_contains(pat, name),
        Pattern::Struct { fields, .. } => fields.values().any(|p| pat_contains(p, name)),
        Pattern::Tuple(elts) => elts.iter().any(|p| pat_contains(p, name)),
        Pattern::Or(alts) => alts.iter().any(|p| pat_contains(p, name)),
        _ => false,
    }
}

fn expr_contains(expr: &Expr, name: &str) -> bool {
    match expr {
        Expr::Variable(path, _) => path.last().map(|p| p == name).unwrap_or(false),
        Expr::Call { func, args, .. } => {
            expr_contains(func, name) || args.iter().any(|a| expr_contains(a, name))
        }
        Expr::Binary { left, right, .. } => {
            expr_contains(left, name) || expr_contains(right, name)
        }
        _ => false,
    }
}

fn location_for(span: &Span, uri: &Url) -> Location {
    let start = Position::new(
        (span.start as u32).saturating_sub(1), // convert byte-offset->line via a mapping in real impl
        0,
    );
    let end = Position::new(start.line, start.character + span.len as u32);
    Location::new(uri.clone(), Range::new(start, end))
}

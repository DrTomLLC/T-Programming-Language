// tlang-lsp/src/handlers/definition.rs

use tower_lsp::lsp_types::*;
use tower_lsp::jsonrpc::Result;
use crate::handlers::hover::span_to_range;
use crate::handlers::diagnostics::Backend;
use shared::tokenizer::tokenize;
use compiler::parser::Parser;
use shared::ast::*;
use std::sync::Arc;

/// Handle `textDocument/definition` requests.
pub async fn handle_definition(
    backend: &Backend,
    params: GotoDefinitionParams,
) -> Result<Option<GotoDefinitionResponse>> {
    let uri = params.text_document_position_params.text_document.uri.clone();
    let pos = params.text_document_position_params.position;

    // Load the current buffer
    let docs = backend.docs.read().await;
    let source = if let Some(src) = docs.get(&uri) {
        src.clone()
    } else {
        return Ok(None);
    };
    drop(docs);

    // Lex + parse entire module
    let tokens = tokenize(&source)
        .map_err(|_| tower_lsp::jsonrpc::Error::invalid_params("lex"))?;
    let mut parser = Parser::from_tokens(tokens.clone())
        .map_err(|_| tower_lsp::jsonrpc::Error::invalid_params("parse"))?;
    let items = parser.parse_module()
        .map_err(|_| tower_lsp::jsonrpc::Error::invalid_params("parse"))?;

    // Compute byte offset
    let byte_offset = {
        let mut line = 0;
        let mut ch = 0;
        for (i, c) in source.char_indices() {
            if (line as u32, ch as u32) == (pos.line, pos.character) {
                break;
            }
            if c == '\n' { line += 1; ch = 0; }
            else { ch += 1; }
        }
        byte_offset.min(source.len())
    };

    // Find token under cursor
    let tok = if let Some(t) = tokens.iter()
        .find(|t| t.span.start <= byte_offset && byte_offset < t.span.start + t.span.len)
    { t } else {
        return Ok(None);
    };

    // Walk AST to find the matching declaration span and URI
    if let Some((decl_uri, decl_span)) =
        find_definition_in_items(&items, &uri, &tok.lexeme)
    {
        let target = Location {
            uri: decl_uri.clone(),
            range: span_to_range(&decl_span),
        };
        Ok(Some(GotoDefinitionResponse::Scalar(target)))
    } else {
        Ok(None)
    }
}

/// Recursively search AST for an `Item` whose name matches `ident`.
/// Returns the URI where it’s declared (we’re single-file for now) and its `Span`.
fn find_definition_in_items(
    items: &[Item],
    current_uri: &Url,
    ident: &str,
) -> Option<(Url, Span)> {
    for item in items {
        match item {
            Item::Function { signature, span, .. } if signature.name == ident => {
                return Some((current_uri.clone(), *span));
            }
            Item::Const { name, span, .. } if name == ident => {
                return Some((current_uri.clone(), *span));
            }
            Item::Static { name, span, .. } if name == ident => {
                return Some((current_uri.clone(), *span));
            }
            Item::Struct { name, span, .. } if name == ident => {
                return Some((current_uri.clone(), *span));
            }
            Item::Enum { name, span, .. } if name == ident => {
                return Some((current_uri.clone(), *span));
            }
            Item::TypeAlias { name, span, .. } if name == ident => {
                return Some((current_uri.clone(), *span));
            }
            // dive into modules, traits, impls, etc.
            Item::Module      { items: inner, .. }
            | Item::Trait     { items: inner, .. }
            | Item::Impl      { items: inner, .. }
            | Item::ExternBlock{ items: inner, .. } => {
                if let Some(found) = find_definition_in_items(inner, current_uri, ident) {
                    return Some(found);
                }
            }
            _ => {}
        }
    }
    None
}

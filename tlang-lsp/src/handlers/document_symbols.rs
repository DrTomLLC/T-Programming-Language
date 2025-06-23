// tlang-lsp/src/handlers/document_symbols.rs

use tower_lsp::lsp_types::*;
use tower_lsp::jsonrpc::Result;
use crate::handlers::diagnostics::Backend;
use shared::tokenizer::tokenize;
use compiler::parser::Parser;
use std::sync::Arc;

/// Handle “textDocument/documentSymbol” requests.
pub async fn handle_document_symbols(
    backend: &Backend,
    params: DocumentSymbolParams,
) -> Result<Option<DocumentSymbolResponse>> {
    let uri = params.text_document.uri.clone();

    // 1) Fetch the source
    let docs = backend.docs.read().await;
    let source = match docs.get(&uri) {
        Some(text) => text.clone(),
        None => return Ok(None),
    };
    drop(docs);

    // 2) Tokenize + parse
    let tokens = match tokenize(&source) {
        Ok(t) => t,
        Err(_) => return Ok(None),
    };
    let mut parser = match Parser::from_tokens(tokens.clone()) {
        Ok(p) => p,
        Err(_) => return Ok(None),
    };
    let items = match parser.parse_module() {
        Ok(stmts) => stmts,
        Err(_) => return Ok(None),
    };

    // 3) Convert AST items into DocumentSymbols
    let mut symbols = Vec::new();
    for item in items {
        if let Some(sym) = to_symbol(&item, &source) {
            symbols.push(sym);
        }
    }

    Ok(Some(DocumentSymbolResponse::Nested(symbols)))
}

/// Recursively map an AST `Item` into a `DocumentSymbol`
fn to_symbol(item: &shared::ast::Item, src: &str) -> Option<DocumentSymbol> {
    use shared::ast::Item::*;
    match item {
        Module { name, items, span, .. } |
        Struct  { name, fields: _, span, .. } |
        Enum    { name, variants: _, span, .. } |
        Trait   { name, items: _, span, .. } |
        ExternBlock { abi: name, items: _, span, .. } |
        TypeAlias { name, .. } |
        Const   { name, .. } |
        Static  { name, .. } => {
            let range = to_range(*span, src);
            // Gather children for nested items (e.g. module contents, impl members)
            let children = match item {
                Module { items, .. } => items.iter().filter_map(|i| to_symbol(i, src)).collect(),
                Trait { items, .. }  => items.iter().filter_map(|i| to_symbol(i, src)).collect(),
                ExternBlock { items, .. } => items.iter().filter_map(|i| to_symbol(i, src)).collect(),
                _ => Vec::new(),
            };
            Some(DocumentSymbol {
                name: name.clone(),
                detail: Some(item_kind(item)),
                kind: symbol_kind(item),
                range,
                selection_range: range,
                children: if children.is_empty() { None } else { Some(children) },
                deprecated: None,
            })
        }
        Function { signature, body, span, .. } => {
            let range = to_range(*span, src);
            let sel = to_range(signature.span, src);
            let children = body.stmts.iter()
                .filter_map(|stmt| match stmt {
                    shared::ast::Stmt::Item(it) => to_symbol(it, src),
                    _ => None,
                })
                .collect();
            Some(DocumentSymbol {
                name: signature.name.clone(),
                detail: Some(format!("fn({}) → {}",
                                     signature.params.iter().map(|p| p.pat.to_string()).collect::<Vec<_>>().join(", "),
                                     signature.return_ty.as_ref().map(|t| t.display()).unwrap_or_default()
                )),
                kind: SymbolKind::FUNCTION,
                range,
                selection_range: sel,
                children: if children.is_empty() { None } else { Some(children) },
                deprecated: None,
            })
        }
        Impl { items, span, .. } => {
            let range = to_range(*span, src);
            let children = items.iter().filter_map(|i| to_symbol(i, src)).collect();
            Some(DocumentSymbol {
                name: "impl".into(),
                detail: None,
                kind: SymbolKind::INTERFACE, // or STRUCT
                range,
                selection_range: range,
                children: if children.is_empty() { None } else { Some(children) },
                deprecated: None,
            })
        }
        _ => None,
    }
}

/// Heuristic: map AST item to `SymbolKind`
fn symbol_kind(item: &shared::ast::Item) -> SymbolKind {
    use shared::ast::Item::*;
    match item {
        Module { .. }      => SymbolKind::MODULE,
        Struct { .. }      => SymbolKind::STRUCT,
        Enum { .. }        => SymbolKind::ENUM,
        Trait { .. }       => SymbolKind::INTERFACE,
        Function { .. }    => SymbolKind::FUNCTION,
        Const { .. }       => SymbolKind::CONSTANT,
        Static { .. }      => SymbolKind::VARIABLE,
        TypeAlias { .. }   => SymbolKind::TYPE_PARAMETER,
        ExternBlock { .. } => SymbolKind::MODULE,
        _                  => SymbolKind::VARIABLE,
    }
}

/// A short description for hover/detail
fn item_kind(item: &shared::ast::Item) -> String {
    use shared::ast::Item::*;
    match item {
        Module { .. }      => "module",
        Struct { .. }      => "struct",
        Enum { .. }        => "enum",
        Trait { .. }       => "trait",
        Function { .. }    => "function",
        Const { .. }       => "const",
        Static { .. }      => "static",
        TypeAlias { .. }   => "type alias",
        ExternBlock { .. } => "extern",
        _                  => "",
    }.to_string()
}

/// Convert our `Span` into an LSP `Range`
fn to_range(span: shared::ast::Span, src: &str) -> Range {
    // naive: count newlines up to `start`, then count columns
    let (line, col) = {
        let mut l = 0;
        let mut c = 0;
        for (i, ch) in src.char_indices() {
            if i == span.start { break; }
            if ch == '\n' { l += 1; c = 0; }
            else { c += 1; }
        }
        (l as u32, c as u32)
    };
    let start = Position::new(line, col);
    // end position: advance by span.len in same line (approx)
    let end = Position::new(line, col + span.len as u32);
    Range::new(start, end)
}

#[tower_lsp::async_trait]
impl tower_lsp::lsp_types::LanguageServer for Backend {
    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>, tower_lsp::jsonrpc::Error> {
        handle_document_symbols(self, params).await
    }
}

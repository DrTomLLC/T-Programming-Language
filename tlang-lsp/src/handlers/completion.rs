// tlang-lsp/src/handlers/completion.rs

use tower_lsp::lsp_types::*;
use tower_lsp::jsonrpc::Result;
use std::collections::HashSet;
use std::sync::Arc;

use shared::tokenizer::tokenize;
use compiler::{parser::Parser, sema};
use crate::handlers::diagnostics::Backend;

/// The fixed list of T‑Lang keywords to always suggest.
const KEYWORDS: &[&str] = &[
    "let", "const", "fn", "if", "else", "while", "loop", "for",
    "return", "match", "struct", "enum", "trait", "impl", "use",
    "async", "await", "move", "pub", "mod", "static", "extern",
];

/// Handle “textDocument/completion” requests.
pub async fn handle_completion(
    backend: &Backend,
    params: CompletionParams,
) -> Result<Option<CompletionResponse>> {
    let uri = &params.text_document_position.text_document.uri;
    let pos = &params.text_document_position.position;

    // 1) Get current buffer
    let source = {
        let docs = backend.docs.read().await;
        if let Some(src) = docs.get(uri) {
            src.clone()
        } else {
            return Ok(None);
        }
    };

    // 2) Tokenize entire file
    let tokens = match tokenize(&source) {
        Ok(t) => t,
        Err(_) => return Ok(None),
    };

    // 3) Build AST + semantic table
    let mut parser = match Parser::from_tokens(tokens.clone()) {
        Ok(p) => p,
        Err(_) => return Ok(None),
    };
    let stmts = match parser.parse_all() {
        Ok(s) => s,
        Err(_) => return Ok(None),
    };
    let mut checker = sema::Checker::new();
    let _ = checker.check(&stmts); // ignore errors here

    // 4) Get all in‑scope names: variables and functions
    let mut names = HashSet::new();
    // a) local & param names from current scopes
    for scope in checker.scopes() {
        for var in &scope.locals {
            names.insert(var.clone());
        }
    }
    // b) all functions
    for func in checker.functions() {
        names.insert(func.name.clone());
    }

    // 5) Build CompletionItems
    let mut items = Vec::new();
    // Keywords
    for &kw in KEYWORDS {
        items.push(CompletionItem {
            label: kw.into(),
            kind: Some(CompletionItemKind::KEYWORD),
            insert_text: Some(format!("{} ", kw)),
            detail: Some("keyword".into()),
            ..Default::default()
        });
    }
    // Scope names
    for name in names {
        items.push(CompletionItem {
            label: name.clone().into(),
            kind: Some(CompletionItemKind::VARIABLE),
            insert_text: Some(name.clone()),
            detail: Some("symbol".into()),
            ..Default::default()
        });
    }

    Ok(Some(CompletionResponse::Array(items)))
}

#[tower_lsp::async_trait]
impl tower_lsp::lsp_types::LanguageServer for Backend {
    async fn completion(
        &self,
        params: CompletionParams,
    ) -> Result<Option<CompletionResponse>, tower_lsp::jsonrpc::Error> {
        handle_completion(self, params).await
    }
}

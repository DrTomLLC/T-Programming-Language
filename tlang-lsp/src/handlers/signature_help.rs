// tlang-lsp/src/handlers/signature_help.rs

use tower_lsp::lsp_types::*;
use tower_lsp::jsonrpc::Result;
use crate::handlers::diagnostics::Backend;
use shared::tokenizer::tokenize;
use compiler::parser::Parser;
use compiler::sema;
use std::sync::Arc;

/// Handle “textDocument/signatureHelp” requests.
pub async fn handle_signature_help(
    backend: &Backend,
    params: SignatureHelpParams,
) -> Result<Option<SignatureHelp>> {
    let uri = params.text_document_position.params.text_document.uri.clone();
    let pos = params.text_document_position.params.position;

    // 1) Fetch current document text
    let docs = backend.docs.read().await;
    let source = if let Some(src) = docs.get(&uri) {
        src.clone()
    } else {
        return Ok(None);
    };
    drop(docs);

    // 2) Tokenize source
    let tokens = match tokenize(&source) {
        Ok(t) => t,
        Err(err) => {
            backend
                .log
                .error(format!("Tokenizer error in signatureHelp: {}", err));
            return Ok(None);
        }
    };

    // 3) Map LSP Position to byte offset
    let offset = match backend
        .offset_map
        .lock()
        .await
        .position_to_offset(&uri, pos)
    {
        Some(off) => off,
        None => return Ok(None),
    };

    // 4) Walk tokens backwards to find the closest `(` before the cursor,
    //    but stop at statement boundary (`;`)
    let mut open_paren_index = None;
    for (i, tok) in tokens.iter().enumerate().rev() {
        let start = tok.span.0;
        let len = tok.span.1;
        if start < offset && tok.kind == shared::token::TokenType::LParen {
            open_paren_index = Some(i);
            break;
        }
        if start + len < offset && tok.kind == shared::token::TokenType::Semicolon {
            break;
        }
    }
    let open_idx = if let Some(idx) = open_paren_index {
        idx
    } else {
        return Ok(None);
    };

    // 5) Identify function name token immediately before `(`
    let fn_name = tokens
        .get(open_idx.checked_sub(1)?)
        .filter(|t| matches!(t.kind, shared::token::TokenType::Identifier))
        .map(|t| t.lexeme.clone());
    let fn_name = if let Some(name) = fn_name {
        name
    } else {
        return Ok(None);
    };

    // 6) Count commas between `(` and cursor to determine active parameter index
    let mut active_param = 0;
    for tok in tokens.iter().skip(open_idx + 1) {
        if tok.span.0 >= offset {
            break;
        }
        if tok.kind == shared::token::TokenType::Comma {
            active_param += 1;
        }
    }

    // 7) Parse and semantically check full file to populate symbol table
    let mut parser = match Parser::from_tokens(tokens.clone()) {
        Ok(p) => p,
        Err(err) => {
            backend
                .log
                .error(format!("Parser init error: {}", err));
            return Ok(None);
        }
    };
    let stmts = match parser.parse_all() {
        Ok(s) => s,
        Err(err) => {
            backend
                .log
                .error(format!("Parser error in signatureHelp: {}", err));
            return Ok(None);
        }
    };
    let mut checker = sema::Checker::new();
    if let Err(err) = checker.check(&stmts) {
        backend
            .log
            .error(format!("Semantic error in signatureHelp: {}", err));
        return Ok(None);
    }

    // 8) Lookup function signature
    let sig = if let Some(func) = checker.lookup_function(&fn_name) {
        func.sig.clone()
    } else {
        return Ok(None);
    };

    // 9) Build LSP SignatureInformation
    let parameters: Vec<ParameterInformation> = sig
        .params
        .iter()
        .map(|param| ParameterInformation {
            label: ParameterLabel::Simple(param.pat.to_string()),
            documentation: param
                .docs
                .as_ref()
                .map(|d| Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: d.clone(),
                })),
        })
        .collect();

    let signature_label = format!(
        "{}({}) -> {}",
        sig.name,
        sig.params
            .iter()
            .map(|p| p.pat.to_string())
            .collect::<Vec<_>>()
            .join(", "),
        sig.return_ty
            .as_ref()
            .map(|t| t.display())
            .unwrap_or_else(|| "()".into())
    );
    let documentation = sig
        .docs
        .clone()
        .map(|d| Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: d,
        }));

    let sig_info = SignatureInformation {
        label: signature_label,
        documentation,
        parameters: Some(parameters),
        active_parameter: Some(active_param as u32),
    };

    Ok(Some(SignatureHelp {
        signatures: vec![sig_info],
        active_signature: Some(0),
        active_parameter: Some(active_param as u32),
    }))
}

#[tower_lsp::async_trait]
impl tower_lsp::lsp_types::LanguageServer for Backend {
    async fn signature_help(
        &self,
        params: SignatureHelpParams,
    ) -> Result<Option<SignatureHelp>, tower_lsp::jsonrpc::Error> {
        handle_signature_help(self, params).await
    }
}

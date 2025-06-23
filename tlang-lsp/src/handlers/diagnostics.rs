// tlang-lsp/src/handlers/diagnostics.rs

use tower_lsp::lsp_types::*;
use tower_lsp::jsonrpc::Result;
use tower_lsp::LanguageServer;
use std::sync::Arc;
use parking_lot::Mutex;

use shared::tokenizer::tokenize;
use compiler::{parser::Parser, sema, codegen};
use errors::TlError;
use miette::SourceSpan;

pub struct Backend {
    /// In‐memory documents
    pub docs: Arc<RwLock<HashMap<Url, String>>>,
    /// Byte offset mapping to LSP positions
    pub offset_map: Arc<Mutex<OffsetMap>>,
    /// LSP client sink for publishing diagnostics
    pub client: tower_lsp::Client,
    // ... any logging or config fields ...
}

impl Backend {
    /// Run full compile pipeline and publish diagnostics for one file.
    pub async fn publish_diagnostics_for(&self, uri: Url) {
        let text = {
            let docs = self.docs.read().await;
            if let Some(src) = docs.get(&uri) {
                src.clone()
            } else {
                return;
            }
        };

        let mut diags = Vec::new();

        // Lexing
        let tokens = match tokenize(&text) {
            Ok(toks) => toks,
            Err(e) => {
                diags.push(self.mk_diagnostic(&uri, &text, e));
                self.client.publish_diagnostics(uri.clone(), diags, None).await;
                return;
            }
        };

        // Parsing
        let mut parser = match Parser::from_tokens(tokens.clone()) {
            Ok(p) => p,
            Err(e) => {
                diags.push(self.mk_diagnostic(&uri, &text, e));
                self.client.publish_diagnostics(uri.clone(), diags, None).await;
                return;
            }
        };
        let stmts = match parser.parse_all() {
            Ok(s) => s,
            Err(e) => {
                diags.push(self.mk_diagnostic(&uri, &text, e));
                self.client.publish_diagnostics(uri.clone(), diags, None).await;
                return;
            }
        };

        // Semantic
        let mut checker = sema::Checker::new();
        if let Err(e) = checker.check(&stmts) {
            diags.push(self.mk_diagnostic(&uri, &text, e));
        }

        // Codegen (optionally catch codegen errors too)
        if let Err(e) = codegen::generate(&stmts) {
            diags.push(self.mk_diagnostic(&uri, &text, e));
        }

        // Publish all collected diagnostics
        self.client.publish_diagnostics(uri.clone(), diags, None).await;
    }

    /// Turn a TlError into an LSP Diagnostic
    fn mk_diagnostic(&self, uri: &Url, src: &str, err: TlError) -> Diagnostic {
        // err.span is a miette::SourceSpan = (offset, len)
        let SourceSpan { offset, len } = err.span;
        let (start, _) = self.offset_map
            .lock()
            .position_to_linecol(uri, offset)
            .unwrap_or((Position::default(), Position::default()));
        let (_, end) = self.offset_map
            .lock()
            .position_to_linecol(uri, offset + len)
            .unwrap_or((Position::default(), Position::default()));

        Diagnostic {
            range: Range { start, end },
            severity: Some(DiagnosticSeverity::Error),
            code: Some(NumberOrString::String(err.code.as_str().into())),
            code_description: None,
            source: Some("tlang".into()),
            message: err.message.clone(),
            related_information: None,
            tags: None,
            data: None,
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        {
            let mut docs = self.docs.write().await;
            docs.insert(uri.clone(), text);
        }
        self.publish_diagnostics_for(uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.into_iter().next() {
            let mut docs = self.docs.write().await;
            docs.insert(uri.clone(), change.text);
        }
        self.publish_diagnostics_for(uri).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.publish_diagnostics_for(params.text_document.uri).await;
    }

    // ... implement other required methods as no‐ops or as needed ...
}

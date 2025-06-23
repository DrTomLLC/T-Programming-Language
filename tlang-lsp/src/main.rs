// This file is part of the Tlang project, which is licensed under the MIT License.
// tlang-lsp/src/main.rs

use shared::ast::{AST, Statement};
use std::{fs, sync::Arc};
use tokio::sync::Mutex;
use tower_lsp::{
    jsonrpc::Result,
    lsp_types::*,
    LspService,
    Server,
};
use tracing;
use crate::utils::{extract_identifier, item_name_and_span, lookup_hover, offset_to_range, position_to_offset};

mod utils;

mod compiler {
    use crate::Statement;

    pub mod parser {
        use crate::Statement;

        pub struct Parser {
            src: String,
        }

        impl Parser {
            pub fn new(src: String) -> Self {
                Self { src }
            }

            pub fn parse(&mut self) -> Result<Vec<Statement>, String> {
                // Simplified implementation for now
                Ok(Vec::new())
            }
        }
    }

    pub fn check(_stmts: &[Statement]) -> Result<(), String> {
        // Simplified implementation for now
        Ok(())
    }

    pub mod codegen {
        pub fn generate() {
            // Simplified implementation for now
        }
    }

    pub mod runtime {
        pub struct Evaluator;
    }
}

use self::compiler::check;
use self::compiler::codegen::generate;
use self::compiler::parser::Parser;
use self::compiler::runtime::Evaluator;

#[derive(Debug)]
struct Backend {
    /// The text of the currently open document.
    text: Mutex<String>,
    /// Parsed AST for that text.
    ast: Mutex<Option<AST>>,
}

#[tower_lsp::async_trait]
impl tower_lsp::LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec![",".into(), "(".into()]),
                    retrigger_characters: Some(vec![",".into()]),
                    work_done_progress_options: Default::default(),
                }),
                ..Default::default()
            },
            server_info: None,
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        tracing::info!("T-Lang LSP initialized");
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        if uri.scheme() == "file" {
            if let Ok(text) = fs::read_to_string(uri.to_file_path().unwrap()) {
                *self.text.lock().await = text.clone();
                let ast = parse_and_check(&text);
                *self.ast.lock().await = ast;
            }
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let text = params.content_changes[0].text.clone();
        *self.text.lock().await = text.clone();
        let ast = parse_and_check(&text);
        *self.ast.lock().await = ast;
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let text = self.text.lock().await.clone();
        let file_ast = self.ast.lock().await;
        let offset = position_to_offset(&text, params.text_document_position_params.position);

        if let Some(ast) = file_ast.as_ref() {
            if let Some(name) = extract_identifier(&text, offset) {
                for item in &ast.items {
                    if let Some((def_name, span)) = item_name_and_span(item) {
                        if def_name == name {
                            let target_range = offset_to_range(&text, span);
                            let loc = Location {
                                uri: params
                                    .text_document_position_params
                                    .text_document
                                    .uri
                                    .clone(),
                                range: target_range,
                            };
                            return Ok(Some(GotoDefinitionResponse::Scalar(loc)));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let text = self.text.lock().await.clone();
        let file_ast = self.ast.lock().await;
        let offset = position_to_offset(&text, params.text_document_position_params.position);

        if let Some(ast) = file_ast.as_ref() {
            if let Some(info) = lookup_hover(ast, offset) {
                let range = offset_to_range(&text, info.span);
                let contents = HoverContents::Scalar(MarkedString::String(info.message));
                return Ok(Some(Hover {
                    contents,
                    range: Some(range),
                }));
            }
        }

        Ok(None)
    }
}

/// Parse the source and run semantic check + codegen to build AST, or return None.
fn parse_and_check(src: &str) -> Option<AST> {
    let mut parser = Parser::new(src.to_string());
    match parser.parse() {
        Ok(stmts) => {
            let ast = AST::new(src.to_string(), stmts.clone(), Vec::new());
            if check(&stmts).is_ok() {
                Some(ast)
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let backend = Backend {
        text: Mutex::new(String::new()),
        ast: Mutex::new(None),
    };

    let (service, socket) = LspService::new(|_client| backend);
    Server::new(stdin, stdout, socket).serve(service).await;
}

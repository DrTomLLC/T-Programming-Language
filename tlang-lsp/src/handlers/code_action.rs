// tlang-lsp/src/handlers/code_action.rs

use tower_lsp::lsp_types::*;
use tower_lsp::jsonrpc::Result;
use crate::handlers::diagnostics::Backend;
use shared::tokenizer::tokenize;
use compiler::parser::Parser;
use shared::ast::*;
use errors::tl::TlError;  // our rich diagnostics type

/// Handle `textDocument/codeAction` requests.
pub async fn handle_code_action(
    backend: &Backend,
    params: CodeActionParams
) -> Result<Option<CodeActionResponse>> {
    let uri = params.text_document.uri.clone();
    let range = params.range;
    let context = params.context;

    // Only respond if there's at least one diagnostic in this range
    let mut actions = Vec::new();
    for diag in &context.diagnostics {
        // We expect our TlError’s code and span in the diagnostic’s data
        if let Some(data) = &diag.data {
            if let Some(err) = backend.err_map.read().await.get(&uri).and_then(|v| v.get(data.as_u64().unwrap() as usize)) {
                // Generate a “Fix” text edit for simple cases (e.g. missing semicolon)
                if err.code == ErrorCode::MissingToken && err.message.contains("`;`") {
                    let fix = WorkspaceEdit {
                        changes: Some({
                            let mut m = std::collections::HashMap::new();
                            m.insert(
                                uri.clone(),
                                vec![TextEdit {
                                    range: Range {
                                        start: Position::new(range.end.line, range.end.character),
                                        end:   Position::new(range.end.line, range.end.character),
                                    },
                                    new_text: ";".to_string(),
                                }],
                            );
                            m
                        }),
                        ..Default::default()
                    };
                    actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                        title: "Insert missing `;`".into(),
                        kind: Some(CodeActionKind::QUICKFIX),
                        diagnostics: Some(vec![diag.clone()]),
                        edit: Some(fix),
                        command: None,
                        is_preferred: Some(true),
                        ..Default::default()
                    }));
                }

                // You can add more fix patterns here for other ErrorCodes...
            }
        }
    }

    if actions.is_empty() {
        Ok(None)
    } else {
        Ok(Some(actions))
    }
}

// tlang-lsp/src/handlers/hover.rs

use tower_lsp::lsp_types::*;
use tower_lsp::jsonrpc::Result;
use crate::handlers::diagnostics::Backend;
use shared::tokenizer::tokenize;
use compiler::parser::Parser;
use shared::ast::*;
use std::sync::Arc;

/// Handle “textDocument/hover” requests.
pub async fn handle_hover(
    backend: &Backend,
    params: HoverParams,
) -> Result<Option<Hover>> {
    let uri = params.text_document_position_params.text_document.uri.clone();
    let pos = params.text_document_position_params.position;

    // 1) Load buffer
    let docs = backend.docs.read().await;
    let source = if let Some(src) = docs.get(&uri) {
        src.clone()
    } else {
        return Ok(None);
    };
    drop(docs);

    // 2) Lex + parse module
    let tokens = tokenize(&source)
        .map_err(|_| tower_lsp::jsonrpc::Error::invalid_params("lex"))?;
    let mut parser = Parser::from_tokens(tokens.clone())
        .map_err(|_| tower_lsp::jsonrpc::Error::invalid_params("parse"))?;
    let items = parser.parse_module()
        .map_err(|_| tower_lsp::jsonrpc::Error::invalid_params("parse"))?;

    // 3) Compute byte offset from (line,character)
    let byte_offset = {
        let mut lineno = 0;
        let mut ch = 0;
        for (i, c) in source.char_indices() {
            if (lineno as u32, ch as u32) == (pos.line, pos.character) {
                break;
            }
            if c == '\n' {
                lineno += 1;
                ch = 0;
            } else {
                ch += 1;
            }
        }
        byte_offset.min(source.len())
    };

    // 4) Find token at cursor
    let tok = if let Some(t) = tokens.iter()
        .find(|t| t.span.start <= byte_offset && byte_offset < t.span.start + t.span.len)
    { t } else {
        return Ok(None);
    };

    // 5) Lookup symbol info (in AST or a symbol table)
    let info = find_symbol_info(&items, &tok.lexeme)
        .unwrap_or_default();

    // 6) Build hover contents
    let contents = HoverContents::Scalar(
        MarkedString::LanguageString(LanguageString {
            language: "tlang".into(),
            value: info,
        })
    );

    let range = span_to_range(&tok.span);

    Ok(Some(Hover { contents, range: Some(range) }))
}

/// Simple walk to find signature or doc-comment for a name.
fn find_symbol_info(items: &[Item], name: &str) -> Option<String> {
    for item in items {
        match item {
            Item::Function { signature, attrs, span } if signature.name == name => {
                // e.g. "fn foo(x: i32) -> i32"
                let sig = format!(
                    "{}fn {}({}) -> {}",
                    docs_from_attrs(attrs),
                    signature.name,
                    signature.params.iter()
                        .map(|p| format!("{}: {:?}", p.pat, p.ty))
                        .collect::<Vec<_>>()
                        .join(", "),
                    signature.return_ty.as_ref().map(|t| format!("{:?}", t)).unwrap_or_else(|| "()".into())
                );
                return Some(sig);
            }
            Item::Const { name: c, ty, attrs, .. } if c == name => {
                return Some(format!("{}const {}: {:?}", docs_from_attrs(attrs), c, ty));
            }
            Item::Static { name: s, ty, attrs, .. } if s == name => {
                return Some(format!("{}static {}: {:?}", docs_from_attrs(attrs), s, ty));
            }
            Item::Struct { name: s, generics, attrs, .. } if s == name => {
                return Some(format!("{}struct {}{:?}", docs_from_attrs(attrs), s, generics));
            }
            Item::Enum { name: e, attrs, .. } if e == name => {
                return Some(format!("{}enum {}", docs_from_attrs(attrs), e));
            }
            // Recurse into modules, impls, etc.
            Item::Module  { items: inner, .. }
            | Item::Trait { items: inner, .. }
            | Item::Impl  { items: inner, .. }
            | Item::ExternBlock { items: inner, .. } => {
                if let Some(info) = find_symbol_info(inner, name) {
                    return Some(info);
                }
            }
            _ => {}
        }
    }
    None
}

/// Pull docs from attributes.
fn docs_from_attrs(attrs: &[Attribute]) -> String {
    attrs.iter().filter_map(|attr| {
        if attr.path == ["doc"] {
            if let Some(MetaItem::NameValue(_, Literal::String(s))) = attr.args.get(0) {
                return Some(s.clone());
            }
        }
        None
    }).map(|s| format!("/// {}\n", s)).collect::<String>()
}

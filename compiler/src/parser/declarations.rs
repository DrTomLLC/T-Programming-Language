// compiler/src/parser/declarations.rs

use crate::parser::{Parser, Result};
use crate::lexer::{Keyword, TokenKind};
use crate::ast::{Declaration, FunctionDecl, StructDecl, FieldDecl, EnumDecl, EnumVariant, UseDecl, Type};

/// Parse any top-level declaration: fn, struct, enum, or use.
pub fn parse_declaration(parser: &mut Parser) -> Result<Declaration> {
    if parser.peek_keyword(Keyword::Fn) {
        let func = parse_function_decl(parser)?;
        Ok(Declaration::Function(func))
    } else if parser.peek_keyword(Keyword::Struct) {
        let strukt = parse_struct_decl(parser)?;
        Ok(Declaration::Struct(strukt))
    } else if parser.peek_keyword(Keyword::Enum) {
        let enm = parse_enum_decl(parser)?;
        Ok(Declaration::Enum(enm))
    } else if parser.peek_keyword(Keyword::Use) {
        let use_decl = parse_use_decl(parser)?;
        Ok(Declaration::Use(use_decl))
    } else {
        Err(parser.error("expected declaration"))
    }
}

/// fn name(params) -> Type? { ... }
pub fn parse_function_decl(parser: &mut Parser) -> Result<FunctionDecl> {
    parser.expect_keyword(Keyword::Fn)?;
    let name = parser.expect_identifier()?;
    parser.expect(TokenKind::OpenParen)?;
    let params = if !parser.peek(TokenKind::CloseParen) {
        parse_param_list(parser)?
    } else {
        Vec::new()
    };
    parser.expect(TokenKind::CloseParen)?;
    let return_type = if parser.peek(TokenKind::Arrow) {
        parser.bump();
        Some(parser.parse_type()?)
    } else {
        None
    };
    let body = parser.parse_block()?;
    Ok(FunctionDecl { name, params, return_type, body })
}

fn parse_param_list(parser: &mut Parser) -> Result<Vec<(String, Type)>> {
    let mut params = Vec::new();
    loop {
        let name = parser.expect_identifier()?;
        parser.expect(TokenKind::Colon)?;
        let ty = parser.parse_type()?;
        params.push((name, ty));
        if parser.peek(TokenKind::Comma) {
            parser.bump();
            continue;
        }
        break;
    }
    Ok(params)
}

/// struct Name { field: Type; ... }
pub fn parse_struct_decl(parser: &mut Parser) -> Result<StructDecl> {
    parser.expect_keyword(Keyword::Struct)?;
    let name = parser.expect_identifier()?;
    parser.expect(TokenKind::OpenBrace)?;
    let mut fields = Vec::new();
    while !parser.peek(TokenKind::CloseBrace) {
        let field_name = parser.expect_identifier()?;
        parser.expect(TokenKind::Colon)?;
        let field_type = parser.parse_type()?;
        parser.expect(TokenKind::Semicolon)?;
        fields.push(FieldDecl { name: field_name, ty: field_type });
    }
    parser.expect(TokenKind::CloseBrace)?;
    Ok(StructDecl { name, fields })
}

/// enum Name { Variant(Type, ...); ... }
pub fn parse_enum_decl(parser: &mut Parser) -> Result<EnumDecl> {
    parser.expect_keyword(Keyword::Enum)?;
    let name = parser.expect_identifier()?;
    parser.expect(TokenKind::OpenBrace)?;
    let mut variants = Vec::new();
    while !parser.peek(TokenKind::CloseBrace) {
        let variant_name = parser.expect_identifier()?;
        let args = if parser.peek(TokenKind::OpenParen) {
            parser.bump();
            let types = parse_type_list(parser)?;
            parser.expect(TokenKind::CloseParen)?;
            types
        } else {
            Vec::new()
        };
        parser.expect(TokenKind::Semicolon)?;
        variants.push(EnumVariant { name: variant_name, types: args });
    }
    parser.expect(TokenKind::CloseBrace)?;
    Ok(EnumDecl { name, variants })
}

/// use path::to::module;
pub fn parse_use_decl(parser: &mut Parser) -> Result<UseDecl> {
    parser.expect_keyword(Keyword::Use)?;
    let path = parse_path(parser)?;
    parser.expect(TokenKind::Semicolon)?;
    Ok(UseDecl { path })
}

fn parse_path(parser: &mut Parser) -> Result<Vec<String>> {
    let mut segments = Vec::new();
    segments.push(parser.expect_identifier()?);
    while parser.peek(TokenKind::ColonColon) {
        parser.bump();
        segments.push(parser.expect_identifier()?);
    }
    Ok(segments)
}

fn parse_type_list(parser: &mut Parser) -> Result<Vec<Type>> {
    let mut types = Vec::new();
    loop {
        types.push(parser.parse_type()?);
        if parser.peek(TokenKind::Comma) {
            parser.bump();
            continue;
        }
        break;
    }
    Ok(types)
}

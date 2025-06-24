//! compiler/src/parser/mod.rs
//! T-Lang parser implementation.
//! Converts token stream into AST using recursive descent parsing.

use shared::{
    Program, Item, Expression, Statement, Type, Block, Literal,
    BinaryOperator, UnaryOperator, FunctionDecl, Parameter,
    StructDecl, EnumDecl, UseDecl, ConstDecl, LetStatement,
    IfStatement, WhileStatement, ReturnStatement, SourceMap,
    BinaryExpression, UnaryExpression, CallExpression,
    TypeKind, PrimitiveType, SafetyLevel, Result, TlError
};
use miette::SourceSpan;

// Use existing lexer from the lexer module
use crate::lexer::{tokenize};
use shared::token::{Token, TokenType as TokenKind};

/// Main parser struct that orchestrates the parsing process
pub struct Parser {
    source: String,
    tokens: Vec<Token>,
    current: usize,
    errors: Vec<TlError>,
}

/// Parser implementation
impl Parser {
    /// Create a new parser for the given source code
    pub fn new(source: String) -> Self {
        // Use existing lexer
        let tokens = match crate::lexer::tokenize(&source) {
            Ok(tokens) => tokens,
            Err(_) => Vec::new(), // Handle error appropriately
        };

        Self {
            source,
            tokens,
            current: 0,
            errors: Vec::new(),
        }
    }

    /// Parse the source code into a Program AST
    pub fn parse(&mut self) -> Result<Program> {
        let mut program = Program::with_source(self.source.clone(), None);

        // Parse all top-level items
        while !self.is_at_end() {
            match self.parse_item() {
                Ok(item) => program.add_item(item),
                Err(err) => {
                    self.errors.push(err);
                    self.synchronize();
                }
            }
        }

        // Return errors if any occurred
        if !self.errors.is_empty() {
            return Err(self.errors[0].clone());
        }

        Ok(program)
    }

    /// Parse a top-level item (function, struct, enum, etc.)
    fn parse_item(&mut self) -> Result<Item> {
        match self.peek_kind() {
            TokenKind::Fn => {
                Ok(Item::Function(self.parse_function()?))
            }
            TokenKind::Struct => {
                Ok(Item::Struct(self.parse_struct()?))
            }
            TokenKind::Enum => {
                Ok(Item::Enum(self.parse_enum()?))
            }
            TokenKind::Use => {
                Ok(Item::Use(self.parse_use()?))
            }
            TokenKind::Const => {
                Ok(Item::Const(self.parse_const()?))
            }
            _ => Err(self.error("Expected item declaration"))
        }
    }

    /// Parse a function declaration
    fn parse_function(&mut self) -> Result<FunctionDecl> {
        let start = self.current_span();
        self.consume(TokenKind::Keyword(Keyword::Fn), "Expected 'fn'")?;

        let name = self.consume_identifier("Expected function name")?;

        self.consume(TokenKind::LeftParen, "Expected '(' after function name")?;
        let mut params = Vec::new();

        // Parse parameters
        if !self.check(TokenKind::RightParen) {
            loop {
                let param_name = self.consume_identifier("Expected parameter name")?;
                self.consume(TokenKind::Colon, "Expected ':' after parameter name")?;
                let param_type = self.parse_type()?;

                params.push(Parameter {
                    name: param_name,
                    ty: param_type,
                    span: self.current_span(),
                });

                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenKind::RightParen, "Expected ')' after parameters")?;

        // Parse return type
        let return_type = if self.match_token(TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        // Parse function body
        let body = self.parse_block()?;
        let end = self.current_span();

        Ok(FunctionDecl {
            name,
            params,
            return_type,
            body,
            span: SourceSpan::new(start.offset().into(), end.offset() - start.offset()),
            safety_level: SafetyLevel::Safe, // TODO: Parse safety annotations
        })
    }

    /// Parse a struct declaration
    fn parse_struct(&mut self) -> Result<StructDecl> {
        let start = self.current_span();
        self.consume(TokenKind::Keyword(Keyword::Struct), "Expected 'struct'")?;

        let name = self.consume_identifier("Expected struct name")?;

        self.consume(TokenKind::LeftBrace, "Expected '{' after struct name")?;
        let mut fields = Vec::new();

        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            let field_name = self.consume_identifier("Expected field name")?;
            self.consume(TokenKind::Colon, "Expected ':' after field name")?;
            let field_type = self.parse_type()?;

            fields.push(shared::FieldDecl {
                name: field_name,
                ty: field_type,
                span: self.current_span(),
            });

            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }

        self.consume(TokenKind::RightBrace, "Expected '}' after struct fields")?;
        let end = self.current_span();

        Ok(StructDecl {
            name,
            fields,
            span: SourceSpan::new(start.offset().into(), end.offset() - start.offset()),
        })
    }

    /// Parse an enum declaration
    fn parse_enum(&mut self) -> Result<EnumDecl> {
        let start = self.current_span();
        self.consume(TokenKind::Keyword(Keyword::Enum), "Expected 'enum'")?;

        let name = self.consume_identifier("Expected enum name")?;

        self.consume(TokenKind::LeftBrace, "Expected '{' after enum name")?;
        let mut variants = Vec::new();

        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            let variant_name = self.consume_identifier("Expected variant name")?;
            let mut types = Vec::new();

            if self.match_token(TokenKind::LeftParen) {
                if !self.check(TokenKind::RightParen) {
                    loop {
                        types.push(self.parse_type()?);
                        if !self.match_token(TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.consume(TokenKind::RightParen, "Expected ')' after variant types")?;
            }

            variants.push(shared::EnumVariant {
                name: variant_name,
                types,
                span: self.current_span(),
            });

            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }

        self.consume(TokenKind::RightBrace, "Expected '}' after enum variants")?;
        let end = self.current_span();

        Ok(EnumDecl {
            name,
            variants,
            span: SourceSpan::new(start.offset().into(), end.offset() - start.offset()),
        })
    }

    /// Parse a use declaration
    fn parse_use(&mut self) -> Result<UseDecl> {
        let start = self.current_span();
        self.consume(TokenKind::Keyword(Keyword::Use), "Expected 'use'")?;

        let mut path = Vec::new();
        path.push(self.consume_identifier("Expected module name")?);

        while self.match_token(TokenKind::ColonColon) {
            path.push(self.consume_identifier("Expected module name after '::'")?);
        }

        self.consume(TokenKind::Semicolon, "Expected ';' after use declaration")?;
        let end = self.current_span();

        Ok(UseDecl {
            path,
            span: SourceSpan::new(start.offset().into(), end.offset() - start.offset()),
        })
    }

    /// Parse a const declaration
    fn parse_const(&mut self) -> Result<ConstDecl> {
        let start = self.current_span();
        self.consume(TokenKind::Keyword(Keyword::Const), "Expected 'const'")?;

        let name = self.consume_identifier("Expected constant name")?;
        self.consume(TokenKind::Colon, "Expected ':' after constant name")?;
        let ty = self.parse_type()?;
        self.consume(TokenKind::Equal, "Expected '=' after constant type")?;
        let value = self.parse_expression()?;
        self.consume(TokenKind::Semicolon, "Expected ';' after constant value")?;
        let end = self.current_span();

        Ok(ConstDecl {
            name,
            ty,
            value,
            span: SourceSpan::new(start.offset().into(), end.offset() - start.offset()),
        })
    }

    /// Parse a block of statements
    fn parse_block(&mut self) -> Result<Block> {
        let start = self.current_span();
        self.consume(TokenKind::LeftBrace, "Expected '{'")?;

        let mut statements = Vec::new();

        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }

        self.consume(TokenKind::RightBrace, "Expected '}'")?;
        let end = self.current_span();

        Ok(Block {
            statements,
            span: SourceSpan::new(start.offset().into(), end.offset() - start.offset()),
        })
    }

    /// Parse a statement
    fn parse_statement(&mut self) -> Result<Statement> {
        match self.peek_kind() {
            TokenKind::Keyword(Keyword::Let) => {
                Ok(Statement::Let(self.parse_let_statement()?))
            }
            TokenKind::Keyword(Keyword::If) => {
                Ok(Statement::If(self.parse_if_statement()?))
            }
            TokenKind::Keyword(Keyword::While) => {
                Ok(Statement::While(self.parse_while_statement()?))
            }
            TokenKind::Keyword(Keyword::Return) => {
                Ok(Statement::Return(self.parse_return_statement()?))
            }
            TokenKind::LeftBrace => {
                Ok(Statement::Block(self.parse_block()?))
            }
            _ => {
                let expr = self.parse_expression()?;
                self.consume(TokenKind::Semicolon, "Expected ';' after expression")?;
                Ok(Statement::Expression(expr))
            }
        }
    }

    /// Parse a let statement
    fn parse_let_statement(&mut self) -> Result<LetStatement> {
        let start = self.current_span();
        self.consume(TokenKind::Keyword(Keyword::Let), "Expected 'let'")?;

        let name = self.consume_identifier("Expected variable name")?;

        let ty = if self.match_token(TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let initializer = if self.match_token(TokenKind::Equal) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.consume(TokenKind::Semicolon, "Expected ';' after let statement")?;
        let end = self.current_span();

        Ok(LetStatement {
            name,
            ty,
            initializer,
            span: SourceSpan::new(start.offset().into(), end.offset() - start.offset()),
        })
    }

    /// Parse an if statement
    fn parse_if_statement(&mut self) -> Result<IfStatement> {
        let start = self.current_span();
        self.consume(TokenKind::Keyword(Keyword::If), "Expected 'if'")?;

        let condition = self.parse_expression()?;
        let then_branch = self.parse_block()?;

        let else_branch = if self.match_token(TokenKind::Keyword(Keyword::Else)) {
            Some(self.parse_block()?)
        } else {
            None
        };

        let end = self.current_span();

        Ok(IfStatement {
            condition,
            then_branch,
            else_branch,
            span: SourceSpan::new(start.offset().into(), end.offset() - start.offset()),
        })
    }

    /// Parse a while statement
    fn parse_while_statement(&mut self) -> Result<WhileStatement> {
        let start = self.current_span();
        self.consume(TokenKind::Keyword(Keyword::While), "Expected 'while'")?;

        let condition = self.parse_expression()?;
        let body = self.parse_block()?;
        let end = self.current_span();

        Ok(WhileStatement {
            condition,
            body,
            span: SourceSpan::new(start.offset().into(), end.offset() - start.offset()),
        })
    }

    /// Parse a return statement
    fn parse_return_statement(&mut self) -> Result<ReturnStatement> {
        let start = self.current_span();
        self.consume(TokenKind::Keyword(Keyword::Return), "Expected 'return'")?;

        let value = if self.check(TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };

        self.consume(TokenKind::Semicolon, "Expected ';' after return statement")?;
        let end = self.current_span();

        Ok(ReturnStatement {
            value,
            span: SourceSpan::new(start.offset().into(), end.offset() - start.offset()),
        })
    }

    /// Parse an expression with full precedence climbing
    fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_assignment()
    }

    /// Parse assignment expressions
    fn parse_assignment(&mut self) -> Result<Expression> {
        let expr = self.parse_logical_or()?;

        if self.match_token(TokenKind::Equal) {
            let operator = BinaryOperator::Assign;
            let right = self.parse_assignment()?;
            return Ok(Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                span: self.current_span(),
            }));
        }

        Ok(expr)
    }

    /// Parse logical OR expressions
    fn parse_logical_or(&mut self) -> Result<Expression> {
        let mut expr = self.parse_logical_and()?;

        while self.match_token(TokenKind::Or) {
            let operator = BinaryOperator::Or;
            let right = self.parse_logical_and()?;
            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                span: self.current_span(),
            });
        }

        Ok(expr)
    }

    /// Parse logical AND expressions
    fn parse_logical_and(&mut self) -> Result<Expression> {
        let mut expr = self.parse_equality()?;

        while self.match_token(TokenKind::And) {
            let operator = BinaryOperator::And;
            let right = self.parse_equality()?;
            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                span: self.current_span(),
            });
        }

        Ok(expr)
    }

    /// Parse equality expressions
    fn parse_equality(&mut self) -> Result<Expression> {
        let mut expr = self.parse_comparison()?;

        while let Some(operator) = self.match_equality_operator() {
            let right = self.parse_comparison()?;
            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                span: self.current_span(),
            });
        }

        Ok(expr)
    }

    /// Parse comparison expressions
    fn parse_comparison(&mut self) -> Result<Expression> {
        let mut expr = self.parse_term()?;

        while let Some(operator) = self.match_comparison_operator() {
            let right = self.parse_term()?;
            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                span: self.current_span(),
            });
        }

        Ok(expr)
    }

    /// Parse additive/subtractive expressions
    fn parse_term(&mut self) -> Result<Expression> {
        let mut expr = self.parse_factor()?;

        while let Some(operator) = self.match_term_operator() {
            let right = self.parse_factor()?;
            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                span: self.current_span(),
            });
        }

        Ok(expr)
    }

    /// Parse multiplicative/divisive expressions
    fn parse_factor(&mut self) -> Result<Expression> {
        let mut expr = self.parse_unary()?;

        while let Some(operator) = self.match_factor_operator() {
            let right = self.parse_unary()?;
            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                span: self.current_span(),
            });
        }

        Ok(expr)
    }

    /// Parse unary expressions
    fn parse_unary(&mut self) -> Result<Expression> {
        if let Some(operator) = self.match_unary_operator() {
            let operand = self.parse_unary()?;
            return Ok(Expression::Unary(UnaryExpression {
                operator,
                operand: Box::new(operand),
                span: self.current_span(),
            }));
        }

        self.parse_call()
    }

    /// Parse function calls and field access
    fn parse_call(&mut self) -> Result<Expression> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(TokenKind::LeftParen) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(TokenKind::Dot) {
                let name = self.consume_identifier("Expected property name after '.'")?;
                expr = Expression::FieldAccess(shared::FieldAccessExpression {
                    object: Box::new(expr),
                    field: name,
                    span: self.current_span(),
                });
            } else if self.match_token(TokenKind::LeftBracket) {
                let index = self.parse_expression()?;
                self.consume(TokenKind::RightBracket, "Expected ']' after array index")?;
                expr = Expression::Index(shared::IndexExpression {
                    object: Box::new(expr),
                    index: Box::new(index),
                    span: self.current_span(),
                });
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// Finish parsing a function call
    fn finish_call(&mut self, callee: Expression) -> Result<Expression> {
        let mut arguments = Vec::new();

        if !self.check(TokenKind::RightParen) {
            loop {
                arguments.push(self.parse_expression()?);
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenKind::RightParen, "Expected ')' after arguments")?;

        Ok(Expression::Call(CallExpression {
            callee: Box::new(callee),
            arguments,
            span: self.current_span(),
        }))
    }

    /// Parse primary expressions (literals, identifiers, grouping)
    fn parse_primary(&mut self) -> Result<Expression> {
        match self.peek_kind() {
            TokenKind::Keyword(Keyword::True) => {
                self.advance();
                Ok(Expression::Literal(Literal::Boolean(true)))
            }
            TokenKind::Keyword(Keyword::False) => {
                self.advance();
                Ok(Expression::Literal(Literal::Boolean(false)))
            }
            TokenKind::Number => {
                let token = self.advance();
                if token.lexeme.contains('.') {
                    let value: f64 = token.lexeme.parse()
                        .map_err(|_| self.error("Invalid float literal"))?;
                    Ok(Expression::Literal(Literal::Float(value)))
                } else {
                    let value: i64 = token.lexeme.parse()
                        .map_err(|_| self.error("Invalid integer literal"))?;
                    Ok(Expression::Literal(Literal::Integer(value)))
                }
            }
            TokenKind::String => {
                let token = self.advance();
                Ok(Expression::Literal(Literal::String(token.lexeme.clone())))
            }
            TokenKind::Character => {
                let token = self.advance();
                let chars: Vec<char> = token.lexeme.chars().collect();
                if chars.len() == 1 {
                    Ok(Expression::Literal(Literal::Character(chars[0])))
                } else {
                    Err(self.error("Invalid character literal"))
                }
            }
            TokenKind::Identifier => {
                let token = self.advance();
                Ok(Expression::Identifier(token.lexeme.clone()))
            }
            TokenKind::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.consume(TokenKind::RightParen, "Expected ')' after expression")?;
                Ok(Expression::Grouping(Box::new(expr)))
            }
            _ => Err(self.error("Expected expression"))
        }
    }

    /// Parse a type annotation
    fn parse_type(&mut self) -> Result<Type> {
        let span = self.current_span();

        match self.peek_kind() {
            TokenKind::Identifier => {
                let name = self.advance().lexeme.clone();
                let kind = match name.as_str() {
                    "bool" => TypeKind::Primitive(PrimitiveType::Bool),
                    "i8" => TypeKind::Primitive(PrimitiveType::I8),
                    "i16" => TypeKind::Primitive(PrimitiveType::I16),
                    "i32" => TypeKind::Primitive(PrimitiveType::I32),
                    "i64" => TypeKind::Primitive(PrimitiveType::I64),
                    "i128" => TypeKind::Primitive(PrimitiveType::I128),
                    "u8" => TypeKind::Primitive(PrimitiveType::U8),
                    "u16" => TypeKind::Primitive(PrimitiveType::U16),
                    "u32" => TypeKind::Primitive(PrimitiveType::U32),
                    "u64" => TypeKind::Primitive(PrimitiveType::U64),
                    "u128" => TypeKind::Primitive(PrimitiveType::U128),
                    "f32" => TypeKind::Primitive(PrimitiveType::F32),
                    "f64" => TypeKind::Primitive(PrimitiveType::F64),
                    "char" => TypeKind::Primitive(PrimitiveType::Char),
                    "str" => TypeKind::Primitive(PrimitiveType::Str),
                    _ => TypeKind::Struct(name), // Assume it's a custom struct
                };
                Ok(Type::new(kind, span))
            }
            _ => Err(self.error("Expected type"))
        }
    }

    // Helper methods for token matching
    fn match_equality_operator(&mut self) -> Option<BinaryOperator> {
        if self.match_token(TokenKind::EqualEqual) {
            Some(BinaryOperator::Equal)
        } else if self.match_token(TokenKind::BangEqual) {
            Some(BinaryOperator::NotEqual)
        } else {
            None
        }
    }

    fn match_comparison_operator(&mut self) -> Option<BinaryOperator> {
        if self.match_token(TokenKind::Greater) {
            Some(BinaryOperator::Greater)
        } else if self.match_token(TokenKind::GreaterEqual) {
            Some(BinaryOperator::GreaterEqual)
        } else if self.match_token(TokenKind::Less) {
            Some(BinaryOperator::Less)
        } else if self.match_token(TokenKind::LessEqual) {
            Some(BinaryOperator::LessEqual)
        } else {
            None
        }
    }

    fn match_term_operator(&mut self) -> Option<BinaryOperator> {
        if self.match_token(TokenKind::Plus) {
            Some(BinaryOperator::Add)
        } else if self.match_token(TokenKind::Minus) {
            Some(BinaryOperator::Sub)
        } else {
            None
        }
    }

    fn match_factor_operator(&mut self) -> Option<BinaryOperator> {
        if self.match_token(TokenKind::Star) {
            Some(BinaryOperator::Mul)
        } else if self.match_token(TokenKind::Slash) {
            Some(BinaryOperator::Div)
        } else if self.match_token(TokenKind::Percent) {
            Some(BinaryOperator::Mod)
        } else {
            None
        }
    }

    fn match_unary_operator(&mut self) -> Option<UnaryOperator> {
        if self.match_token(TokenKind::Bang) {
            Some(UnaryOperator::Not)
        } else if self.match_token(TokenKind::Minus) {
            Some(UnaryOperator::Minus)
        } else if self.match_token(TokenKind::Ampersand) {
            Some(UnaryOperator::Reference)
        } else if self.match_token(TokenKind::Star) {
            Some(UnaryOperator::Dereference)
        } else {
            None
        }
    }

    // Token stream management
    fn is_at_end(&self) -> bool {
        self.peek_kind() == TokenKind::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn peek_kind(&self) -> TokenKind {
        self.peek().kind.clone()
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        &self.tokens[self.current - 1]
    }

    fn check(&self, kind: TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek_kind() == kind
        }
    }

    fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume(&mut self, kind: TokenKind, message: &str) -> Result<&Token> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err(self.error(message))
        }
    }

    fn consume_identifier(&mut self, message: &str) -> Result<String> {
        if self.check(TokenKind::Identifier) {
            Ok(self.advance().lexeme.clone())
        } else {
            Err(self.error(message))
        }
    }

    fn current_span(&self) -> SourceSpan {
        let token = self.peek();
        SourceSpan::new(token.span.start.into(), token.span.len)
    }

    fn error(&self, message: &str) -> TlError {
        TlError::parser(
            self.source.clone(),
            self.current_span(),
            message.to_string(),
        )
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.tokens[self.current - 1].kind == TokenKind::Semicolon {
                return;
            }

            match self.peek_kind() {
                TokenKind::Keyword(Keyword::Fn) |
                TokenKind::Keyword(Keyword::Struct) |
                TokenKind::Keyword(Keyword::Enum) |
                TokenKind::Keyword(Keyword::Let) |
                TokenKind::Keyword(Keyword::If) |
                TokenKind::Keyword(Keyword::While) |
                TokenKind::Keyword(Keyword::Return) => return,
                _ => {}
            }

            self.advance();
        }
    }
}

// Public convenience functions
pub fn parse_source(source: &str) -> Result<Program> {
    let mut parser = Parser::new(source.to_string());
    parser.parse()
}

pub fn parse_expression(source: &str) -> Result<Expression> {
    let mut parser = Parser::new(source.to_string());
    parser.parse_expression()
}

// Keywords definition
#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    Fn,
    Let,
    If,
    Else,
    While,
    Return,
    Struct,
    Enum,
    Use,
    Const,
    True,
    False,
}

// Token types (these should be defined in lexer.rs)
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Number,
    String,
    Character,
    Identifier,

    // Keywords
    Keyword(Keyword),

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    EqualEqual,
    Bang,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
    Ampersand,

    // Punctuation
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Semicolon,
    Colon,
    ColonColon,
    Dot,
    Arrow,

    // Special
    Eof,
}
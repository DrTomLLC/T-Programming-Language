//! compiler/src/types/mod.rs
//! Type checking and inference for T-Lang.
//! Ensures type safety and performs type inference on the AST.

use shared::{
    Program, Item, Expression, Statement, Type, TypeKind, PrimitiveType,
    FunctionDecl, Parameter, Block, BinaryOperator, UnaryOperator,
    Literal, LetStatement, IfStatement, WhileStatement, ReturnStatement,
    BinaryExpression, UnaryExpression, CallExpression,
    FieldAccessExpression, IndexExpression, SafetyLevel, Result, TlError
};
use miette::SourceSpan;
use std::collections::HashMap;

pub mod checker;

/// Type checker for T-Lang programs
pub struct TypeChecker {
    source: String,
    functions: HashMap<String, FunctionSignature>,
    variables: HashMap<String, Type>,
    structs: HashMap<String, StructInfo>,
    enums: HashMap<String, EnumInfo>,
    current_function: Option<String>,
    return_type: Option<Type>,
    errors: Vec<TlError>,
}

/// Function signature information
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub params: Vec<Type>,
    pub return_type: Type,
    pub safety_level: SafetyLevel,
}

/// Struct type information
#[derive(Debug, Clone)]
pub struct StructInfo {
    pub fields: HashMap<String, Type>,
}

/// Enum type information
#[derive(Debug, Clone)]
pub struct EnumInfo {
    pub variants: HashMap<String, Vec<Type>>,
}

impl TypeChecker {
    /// Create a new type checker
    pub fn new(source: String) -> Self {
        let mut checker = Self {
            source,
            functions: HashMap::new(),
            variables: HashMap::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            current_function: None,
            return_type: None,
            errors: Vec::new(),
        };

        checker.add_builtin_functions();
        checker
    }

    /// Type check a complete program
    pub fn check_program(&mut self, program: &mut Program) -> Result<()> {
        // First pass: collect declarations
        for item in &program.items {
            match item {
                Item::Function(func) => {
                    self.collect_function_signature(func)?;
                }
                Item::Struct(struct_decl) => {
                    self.collect_struct_info(struct_decl)?;
                }
                Item::Enum(enum_decl) => {
                    self.collect_enum_info(enum_decl)?;
                }
                _ => {} // Skip other items for now
            }
        }

        // Second pass: type check implementations
        for item in &program.items {
            match item {
                Item::Function(func) => {
                    self.check_function(func)?;
                }
                _ => {} // Other items don't need detailed checking yet
            }
        }

        if !self.errors.is_empty() {
            return Err(self.errors[0].clone());
        }

        Ok(())
    }

    /// Collect function signature for type checking
    fn collect_function_signature(&mut self, func: &FunctionDecl) -> Result<()> {
        let param_types: Vec<Type> = func.params.iter().map(|p| p.ty.clone()).collect();
        let return_type = func.return_type.clone().unwrap_or_else(|| {
            Type::primitive(PrimitiveType::Unit, SourceSpan::new(0.into(), 0))
        });

        let signature = FunctionSignature {
            params: param_types,
            return_type,
            safety_level: func.safety_level,
        };

        self.functions.insert(func.name.clone(), signature);
        Ok(())
    }

    /// Collect struct information
    fn collect_struct_info(&mut self, struct_decl: &shared::StructDecl) -> Result<()> {
        let mut fields = HashMap::new();
        for field in &struct_decl.fields {
            fields.insert(field.name.clone(), field.ty.clone());
        }

        self.structs.insert(struct_decl.name.clone(), StructInfo { fields });
        Ok(())
    }

    /// Collect enum information
    fn collect_enum_info(&mut self, enum_decl: &shared::EnumDecl) -> Result<()> {
        let mut variants = HashMap::new();
        for variant in &enum_decl.variants {
            variants.insert(variant.name.clone(), variant.types.clone());
        }

        self.enums.insert(enum_decl.name.clone(), EnumInfo { variants });
        Ok(())
    }

    /// Type check a function
    fn check_function(&mut self, func: &FunctionDecl) -> Result<()> {
        self.current_function = Some(func.name.clone());
        self.return_type = func.return_type.clone();

        // Add parameters to variable scope
        let old_variables = self.variables.clone();
        for param in &func.params {
            self.variables.insert(param.name.clone(), param.ty.clone());
        }

        // Check function body
        let result = self.check_block(&func.body);

        // Restore variable scope
        self.variables = old_variables;
        self.current_function = None;
        self.return_type = None;

        result
    }

    /// Type check a block of statements
    fn check_block(&mut self, block: &Block) -> Result<()> {
        for statement in &block.statements {
            self.check_statement(statement)?;
        }
        Ok(())
    }

    /// Type check a statement
    fn check_statement(&mut self, statement: &Statement) -> Result<()> {
        match statement {
            Statement::Expression(expr) => {
                self.check_expression(expr)?;
                Ok(())
            }
            Statement::Let(let_stmt) => self.check_let_statement(let_stmt),
            Statement::If(if_stmt) => self.check_if_statement(if_stmt),
            Statement::While(while_stmt) => self.check_while_statement(while_stmt),
            Statement::Return(return_stmt) => self.check_return_statement(return_stmt),
            Statement::Block(block) => self.check_block(block),
        }
    }

    /// Type check a let statement
    fn check_let_statement(&mut self, let_stmt: &LetStatement) -> Result<()> {
        let var_type = if let Some(ref initializer) = let_stmt.initializer {
            let init_type = self.check_expression(initializer)?;

            if let Some(ref declared_type) = let_stmt.ty {
                if !self.types_compatible(declared_type, &init_type) {
                    return Err(TlError::Type {
                        src: self.source.clone(),
                        span: let_stmt.span,
                        message: format!(
                            "Type mismatch: declared type {:?} but initializer has type {:?}",
                            declared_type.kind, init_type.kind
                        ),
                    });
                }
                declared_type.clone()
            } else {
                init_type
            }
        } else if let Some(ref declared_type) = let_stmt.ty {
            declared_type.clone()
        } else {
            return Err(TlError::Type {
                src: self.source.clone(),
                span: let_stmt.span,
                message: "Variable must have either type annotation or initializer".to_string(),
            });
        };

        self.variables.insert(let_stmt.name.clone(), var_type);
        Ok(())
    }

    /// Type check an if statement
    fn check_if_statement(&mut self, if_stmt: &IfStatement) -> Result<()> {
        let condition_type = self.check_expression(&if_stmt.condition)?;

        if !self.is_boolean_type(&condition_type) {
            return Err(TlError::Type {
                src: self.source.clone(),
                span: if_stmt.span,
                message: format!("If condition must be boolean, found {:?}", condition_type.kind),
            });
        }

        self.check_block(&if_stmt.then_branch)?;

        if let Some(ref else_branch) = if_stmt.else_branch {
            self.check_block(else_branch)?;
        }

        Ok(())
    }

    /// Type check a while statement
    fn check_while_statement(&mut self, while_stmt: &WhileStatement) -> Result<()> {
        let condition_type = self.check_expression(&while_stmt.condition)?;

        if !self.is_boolean_type(&condition_type) {
            return Err(TlError::Type {
                src: self.source.clone(),
                span: while_stmt.span,
                message: format!("While condition must be boolean, found {:?}", condition_type.kind),
            });
        }

        self.check_block(&while_stmt.body)
    }

    /// Type check a return statement
    fn check_return_statement(&mut self, return_stmt: &ReturnStatement) -> Result<()> {
        let actual_type = if let Some(ref value) = return_stmt.value {
            self.check_expression(value)?
        } else {
            Type::primitive(PrimitiveType::Unit, return_stmt.span)
        };

        let expected_type = self.return_type.clone().unwrap_or_else(|| {
            Type::primitive(PrimitiveType::Unit, SourceSpan::new(0.into(), 0))
        });

        if !self.types_compatible(&expected_type, &actual_type) {
            return Err(TlError::Type {
                src: self.source.clone(),
                span: return_stmt.span,
                message: format!(
                    "Return type mismatch: expected {:?}, found {:?}",
                    expected_type.kind, actual_type.kind
                ),
            });
        }

        Ok(())
    }

    /// Type check an expression and return its type
    fn check_expression(&mut self, expression: &Expression) -> Result<Type> {
        match expression {
            Expression::Literal(literal) => Ok(self.check_literal(literal)),
            Expression::Identifier(name) => self.check_identifier(name),
            Expression::Binary(binary) => self.check_binary_expression(binary),
            Expression::Unary(unary) => self.check_unary_expression(unary),
            Expression::Call(call) => self.check_call_expression(call),
            Expression::FieldAccess(field_access) => self.check_field_access(field_access),
            Expression::Index(index) => self.check_index_expression(index),
            Expression::Grouping(expr) => self.check_expression(expr),
        }
    }

    /// Get the type of a literal
    fn check_literal(&self, literal: &Literal) -> Type {
        let span = SourceSpan::new(0.into(), 0); // TODO: Get proper span
        match literal {
            Literal::Boolean(_) => Type::primitive(PrimitiveType::Bool, span),
            Literal::Integer(_) => Type::primitive(PrimitiveType::I32, span),
            Literal::Float(_) => Type::primitive(PrimitiveType::F64, span),
            Literal::String(_) => Type::primitive(PrimitiveType::Str, span),
            Literal::Character(_) => Type::primitive(PrimitiveType::Char, span),
            Literal::Unit => Type::primitive(PrimitiveType::Unit, span),
        }
    }

    /// Type check an identifier
    fn check_identifier(&self, name: &str) -> Result<Type> {
        if let Some(var_type) = self.variables.get(name) {
            Ok(var_type.clone())
        } else {
            Err(TlError::Type {
                src: self.source.clone(),
                span: SourceSpan::new(0.into(), 0), // TODO: Get proper span
                message: format!("Undefined variable: {}", name),
            })
        }
    }

    /// Type check a binary expression
    fn check_binary_expression(&mut self, binary: &BinaryExpression) -> Result<Type> {
        let left_type = self.check_expression(&binary.left)?;
        let right_type = self.check_expression(&binary.right)?;

        match binary.operator {
            BinaryOperator::Add | BinaryOperator::Sub | BinaryOperator::Mul | BinaryOperator::Div | BinaryOperator::Mod => {
                if self.is_numeric_type(&left_type) && self.types_compatible(&left_type, &right_type) {
                    Ok(left_type)
                } else {
                    Err(TlError::Type {
                        src: self.source.clone(),
                        span: binary.span,
                        message: format!(
                            "Arithmetic operation requires compatible numeric types, found {:?} and {:?}",
                            left_type.kind, right_type.kind
                        ),
                    })
                }
            }
            BinaryOperator::Equal | BinaryOperator::NotEqual => {
                if self.types_compatible(&left_type, &right_type) {
                    Ok(Type::primitive(PrimitiveType::Bool, binary.span))
                } else {
                    Err(TlError::Type {
                        src: self.source.clone(),
                        span: binary.span,
                        message: format!(
                            "Cannot compare types {:?} and {:?}",
                            left_type.kind, right_type.kind
                        ),
                    })
                }
            }
            BinaryOperator::Less | BinaryOperator::LessEqual | BinaryOperator::Greater | BinaryOperator::GreaterEqual => {
                if self.is_numeric_type(&left_type) && self.types_compatible(&left_type, &right_type) {
                    Ok(Type::primitive(PrimitiveType::Bool, binary.span))
                } else {
                    Err(TlError::Type {
                        src: self.source.clone(),
                        span: binary.span,
                        message: format!(
                            "Comparison requires compatible numeric types, found {:?} and {:?}",
                            left_type.kind, right_type.kind
                        ),
                    })
                }
            }
            BinaryOperator::And | BinaryOperator::Or => {
                if self.is_boolean_type(&left_type) && self.is_boolean_type(&right_type) {
                    Ok(Type::primitive(PrimitiveType::Bool, binary.span))
                } else {
                    Err(TlError::Type {
                        src: self.source.clone(),
                        span: binary.span,
                        message: "Logical operations require boolean operands".to_string(),
                    })
                }
            }
            BinaryOperator::Assign => {
                if self.types_compatible(&left_type, &right_type) {
                    Ok(right_type)
                } else {
                    Err(TlError::Type {
                        src: self.source.clone(),
                        span: binary.span,
                        message: format!(
                            "Cannot assign {:?} to {:?}",
                            right_type.kind, left_type.kind
                        ),
                    })
                }
            }
        }
    }

    /// Type check a unary expression
    fn check_unary_expression(&mut self, unary: &UnaryExpression) -> Result<Type> {
        let operand_type = self.check_expression(&unary.operand)?;

        match unary.operator {
            UnaryOperator::Minus => {
                if self.is_numeric_type(&operand_type) {
                    Ok(operand_type)
                } else {
                    Err(TlError::Type {
                        src: self.source.clone(),
                        span: unary.span,
                        message: format!("Unary minus requires numeric type, found {:?}", operand_type.kind),
                    })
                }
            }
            UnaryOperator::Not => {
                if self.is_boolean_type(&operand_type) {
                    Ok(operand_type)
                } else {
                    Err(TlError::Type {
                        src: self.source.clone(),
                        span: unary.span,
                        message: "Logical not requires boolean type".to_string(),
                    })
                }
            }
            UnaryOperator::Reference => {
                // TODO: Implement proper reference types
                Ok(operand_type)
            }
            UnaryOperator::Dereference => {
                // TODO: Implement proper pointer/reference types
                Ok(operand_type)
            }
        }
    }

    /// Type check a function call
    fn check_call_expression(&mut self, call: &CallExpression) -> Result<Type> {
        // For now, assume callee is an identifier
        if let Expression::Identifier(func_name) = &*call.callee {
            if let Some(signature) = self.functions.get(func_name) {
                // Check argument count
                if call.arguments.len() != signature.params.len() {
                    return Err(TlError::Type {
                        src: self.source.clone(),
                        span: call.span,
                        message: format!(
                            "Function {} expects {} arguments, got {}",
                            func_name, signature.params.len(), call.arguments.len()
                        ),
                    });
                }

                // Check argument types
                for (i, (arg, expected_type)) in call.arguments.iter().zip(&signature.params).enumerate() {
                    let arg_type = self.check_expression(arg)?;
                    if !self.types_compatible(expected_type, &arg_type) {
                        return Err(TlError::Type {
                            src: self.source.clone(),
                            span: call.span,
                            message: format!(
                                "Argument {} to function {}: expected {:?}, found {:?}",
                                i + 1, func_name, expected_type.kind, arg_type.kind
                            ),
                        });
                    }
                }

                Ok(signature.return_type.clone())
            } else {
                Err(TlError::Type {
                    src: self.source.clone(),
                    span: call.span,
                    message: format!("Undefined function: {}", func_name),
                })
            }
        } else {
            Err(TlError::Type {
                src: self.source.clone(),
                span: call.span,
                message: "Function calls on non-identifier expressions not yet supported".to_string(),
            })
        }
    }

    /// Type check field access
    fn check_field_access(&mut self, field_access: &FieldAccessExpression) -> Result<Type> {
        let object_type = self.check_expression(&field_access.object)?;

        if let TypeKind::Struct(struct_name) = &object_type.kind {
            if let Some(struct_info) = self.structs.get(struct_name) {
                if let Some(field_type) = struct_info.fields.get(&field_access.field) {
                    Ok(field_type.clone())
                } else {
                    Err(TlError::Type {
                        src: self.source.clone(),
                        span: field_access.span,
                        message: format!(
                            "Struct {} has no field named {}",
                            struct_name, field_access.field
                        ),
                    })
                }
            } else {
                Err(TlError::Type {
                    src: self.source.clone(),
                    span: field_access.span,
                    message: format!("Unknown struct type: {}", struct_name),
                })
            }
        } else {
            Err(TlError::Type {
                src: self.source.clone(),
                span: field_access.span,
                message: format!("Cannot access field on non-struct type: {:?}", object_type.kind),
            })
        }
    }

    /// Type check array/slice indexing
    fn check_index_expression(&mut self, index: &IndexExpression) -> Result<Type> {
        let object_type = self.check_expression(&index.object)?;
        let index_type = self.check_expression(&index.index)?;

        // Check that index is an integer
        if !self.is_integer_type(&index_type) {
            return Err(TlError::Type {
                src: self.source.clone(),
                span: index.span,
                message: format!("Array index must be integer, found {:?}", index_type.kind),
            });
        }

        // Extract element type from array/slice
        match &object_type.kind {
            TypeKind::Array(element_type, _) => Ok((**element_type).clone()),
            TypeKind::Slice(element_type) => Ok((**element_type).clone()),
            _ => Err(TlError::Type {
                src: self.source.clone(),
                span: index.span,
                message: format!("Cannot index into type: {:?}", object_type.kind),
            }),
        }
    }

    // Helper methods for type checking

    /// Check if a type is boolean
    fn is_boolean_type(&self, ty: &Type) -> bool {
        matches!(ty.kind, TypeKind::Primitive(PrimitiveType::Bool))
    }

    /// Check if a type is numeric
    fn is_numeric_type(&self, ty: &Type) -> bool {
        matches!(ty.kind, TypeKind::Primitive(prim) if prim.is_numeric())
    }

    /// Check if a type is an integer
    fn is_integer_type(&self, ty: &Type) -> bool {
        matches!(ty.kind, TypeKind::Primitive(prim) if prim.is_integer())
    }

    /// Check if two types are compatible
    fn types_compatible(&self, a: &Type, b: &Type) -> bool {
        // For now, require exact type equality
        // TODO: Implement proper type compatibility rules (subtyping, coercion, etc.)
        a.kind == b.kind
    }

    /// Add built-in functions to the function registry
    fn add_builtin_functions(&mut self) {
        // Add print function
        self.functions.insert("print".to_string(), FunctionSignature {
            params: vec![Type::primitive(PrimitiveType::Str, SourceSpan::new(0.into(), 0))],
            return_type: Type::primitive(PrimitiveType::Unit, SourceSpan::new(0.into(), 0)),
            safety_level: SafetyLevel::Safe,
        });
    }
}

// Helper trait for primitive type checking
impl PrimitiveType {
    pub fn is_numeric(self) -> bool {
        matches!(self,
            PrimitiveType::I8 | PrimitiveType::I16 | PrimitiveType::I32 | PrimitiveType::I64 | PrimitiveType::I128 |
            PrimitiveType::U8 | PrimitiveType::U16 | PrimitiveType::U32 | PrimitiveType::U64 | PrimitiveType::U128 |
            PrimitiveType::F32 | PrimitiveType::F64
        )
    }

    pub fn is_integer(self) -> bool {
        matches!(self,
            PrimitiveType::I8 | PrimitiveType::I16 | PrimitiveType::I32 | PrimitiveType::I64 | PrimitiveType::I128 |
            PrimitiveType::U8 | PrimitiveType::U16 | PrimitiveType::U32 | PrimitiveType::U64 | PrimitiveType::U128
        )
    }
}

// Public convenience functions
pub fn check_program(program: &mut Program) -> Result<()> {
    let source = program.source_map.source.clone();
    let mut checker = TypeChecker::new(source);
    checker.check_program(program)
}

pub fn check_expression(expression: &Expression, source: String) -> Result<Type> {
    let mut checker = TypeChecker::new(source);
    checker.check_expression(expression)
}
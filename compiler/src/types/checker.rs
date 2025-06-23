// compiler/src/types/checker.rs
//! Type checker for T-Lang.
//! Performs type inference and checking with safety analysis for critical systems.

use shared::{
    Program, Item, ItemKind, Stmt, StmtKind, Expr, ExprKind, Type, TypeKind,
    PrimitiveType, BinaryOp, UnaryOp, Literal, Pattern, PatternKind,
    Result, TlError, stmt
};
use miette::SourceSpan;
use std::collections::HashMap;

/// Type checking context with symbol tables and inference state.
pub struct TypeChecker {
    /// Current scope's variable types
    variables: HashMap<String, Type>,
    /// Function signatures
    functions: HashMap<String, FunctionSignature>,
    /// Type definitions (structs, enums, aliases)
    types: HashMap<String, TypeDefinition>,
    /// Type inference variable counter
    next_type_var: u32,
    /// Active type constraints for inference
    constraints: Vec<TypeConstraint>,
    /// Source code for error reporting
    source: String,
}

/// Function signature information.
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub params: Vec<Type>,
    pub return_type: Type,
    pub safety_level: shared::SafetyLevel,
}

/// Type definition for user-defined types.
#[derive(Debug, Clone)]
pub enum TypeDefinition {
    Struct { fields: HashMap<String, Type> },
    Enum { variants: HashMap<String, Vec<Type>> },
    Alias { target: Type },
}

/// Type constraint for inference.
#[derive(Debug, Clone)]
pub struct TypeConstraint {
    pub left: Type,
    pub right: Type,
    pub span: SourceSpan,
    pub reason: String,
}

impl TypeChecker {
    /// Create a new type checker with built-in types.
    pub fn new(source: String) -> Self {
        let mut checker = Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            types: HashMap::new(),
            next_type_var: 0,
            constraints: Vec::new(),
            source,
        };

        checker.add_builtin_functions();
        checker
    }

    /// Type check a complete program.
    pub fn check_program(&mut self, program: &mut Program) -> Result<()> {
        // First pass: collect type definitions and function signatures
        for item in &program.items {
            self.collect_item_signature(item)?;
        }

        // Second pass: type check all items
        for item in &mut program.items {
            self.check_item(item)?;
        }

        // Solve type inference constraints
        self.solve_constraints()?;

        Ok(())
    }

    /// Collect type information from items without checking bodies.
    fn collect_item_signature(&mut self, item: &Item) -> Result<()> {
        match &item.kind {
            ItemKind::Function { name, params, return_type, .. } => {
                let param_types: Result<Vec<Type>> = params.iter()
                    .map(|p| Ok(p.ty.clone()))
                    .collect();

                let ret_type = return_type.clone().unwrap_or_else(|| {
                    Type::new(TypeKind::Primitive(PrimitiveType::Unit), item.span)
                });

                self.functions.insert(name.clone(), FunctionSignature {
                    params: param_types?,
                    return_type: ret_type,
                    safety_level: shared::SafetyLevel::Safe,
                });
            }

            ItemKind::Struct { name, fields, .. } => {
                match fields {
                    shared::stmt::StructFields::Named(field_list) => {
                        let field_map: HashMap<String, Type> = field_list.iter()
                            .map(|f| (f.name.clone(), f.ty.clone()))
                            .collect();

                        self.types.insert(name.clone(), TypeDefinition::Struct {
                            fields: field_map,
                        });
                    }
                    _ => {} // Handle other field types as needed
                }
            }

            ItemKind::TypeAlias { name, ty, .. } => {
                self.types.insert(name.clone(), TypeDefinition::Alias {
                    target: ty.clone(),
                });
            }

            _ => {} // Other items don't contribute to type environment
        }

        Ok(())
    }

    /// Type check a top-level item.
    fn check_item(&mut self, item: &mut Item) -> Result<()> {
        match &mut item.kind {
            ItemKind::Function { params, body, return_type, .. } => {
                // Enter function scope
                self.push_scope();

                // Add parameters to scope
                for param in params {
                    if let PatternKind::Ident(name) = &param.pattern.kind {
                        self.variables.insert(name.clone(), param.ty.clone());
                    }
                }

                // Type check body
                if let Some(body_expr) = body {
                    let body_type = self.check_expr(body_expr)?;

                    // Check return type compatibility
                    if let Some(expected_return) = return_type {
                        self.require_compatible(&body_type, expected_return, body_expr.span,
                                                "Function body type doesn't match return type")?;
                    }
                }

                self.pop_scope();
            }

            ItemKind::Const { ty, value, .. } => {
                let value_type = self.check_expr(value)?;
                self.require_compatible(&value_type, ty, value.span,
                                        "Constant value type doesn't match declared type")?;
            }

            ItemKind::Static { ty, value, .. } => {
                let value_type = self.check_expr(value)?;
                self.require_compatible(&value_type, ty, value.span,
                                        "Static value type doesn't match declared type")?;
            }

            _ => {} // Other items don't need body checking
        }

        Ok(())
    }

    /// Type check an expression and return its type.
    pub fn check_expr(&mut self, expr: &mut Expr) -> Result<Type> {
        let expr_type = match &mut expr.kind {
            ExprKind::Literal(literal) => self.check_literal(literal),

            ExprKind::Variable { path } => self.check_variable(path, expr.span),

            ExprKind::Binary { left, op, right } => {
                self.check_binary_expr(left, op, right, expr.span)
            }

            ExprKind::Unary { op, expr: inner } => {
                self.check_unary_expr(op, inner, expr.span)
            }

            ExprKind::Call { callee, args, .. } => {
                self.check_call_expr(callee, args, expr.span)
            }

            ExprKind::If { condition, then_branch, else_branch } => {
                self.check_if_expr(condition, then_branch, else_branch, expr.span)
            }

            ExprKind::Block(block) => {
                self.check_block_expr(block)
            }

            ExprKind::Assign { target, value, .. } => {
                self.check_assign_expr(target, value, expr.span)
            }

            _ => {
                return Err(TlError::type_error(
                    self.source.clone(),
                    expr.span,
                    format!("Type checking not implemented for: {:?}", expr.kind),
                ));
            }
        }?;

        // Store the inferred type in the expression
        expr.ty = Some(expr_type.clone());

        Ok(expr_type)
    }

    /// Type check a literal.
    fn check_literal(&self, literal: &Literal) -> Result<Type> {
        let type_kind = match literal {
            Literal::Integer(_) => TypeKind::Primitive(PrimitiveType::I32), // Default to i32
            Literal::Float(_) => TypeKind::Primitive(PrimitiveType::F64),   // Default to f64
            Literal::String(_) => TypeKind::Primitive(PrimitiveType::Str),
            Literal::Char(_) => TypeKind::Primitive(PrimitiveType::Char),
            Literal::Bool(_) => TypeKind::Primitive(PrimitiveType::Bool),
            Literal::Unit => TypeKind::Primitive(PrimitiveType::Unit),
        };

        Ok(Type::new(type_kind, SourceSpan::new(0.into(), 0)))
    }

    /// Type check a variable reference.
    fn check_variable(&self, path: &[String], span: SourceSpan) -> Result<Type> {
        if path.len() == 1 {
            let name = &path[0];
            if let Some(var_type) = self.variables.get(name) {
                Ok(var_type.clone())
            } else {
                Err(TlError::type_error(
                    self.source.clone(),
                    span,
                    format!("Undefined variable: {}", name),
                ))
            }
        } else {
            Err(TlError::type_error(
                self.source.clone(),
                span,
                "Module paths not yet implemented".to_string(),
            ))
        }
    }

    /// Type check a binary expression.
    fn check_binary_expr(&mut self, left: &mut Expr, op: &BinaryOp, right: &mut Expr, span: SourceSpan) -> Result<Type> {
        let left_type = self.check_expr(left)?;
        let right_type = self.check_expr(right)?;

        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                // Arithmetic operations require numeric types
                self.require_numeric(&left_type, left.span)?;
                self.require_numeric(&right_type, right.span)?;
                self.require_compatible(&left_type, &right_type, span, "Arithmetic operands must have compatible types")?;
                Ok(left_type)
            }

            BinaryOp::Eq | BinaryOp::Ne => {
                // Equality requires compatible types
                self.require_compatible(&left_type, &right_type, span, "Equality operands must have compatible types")?;
                Ok(Type::new(TypeKind::Primitive(PrimitiveType::Bool), span))
            }

            BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                // Comparison requires numeric types
                self.require_numeric(&left_type, left.span)?;
                self.require_numeric(&right_type, right.span)?;
                self.require_compatible(&left_type, &right_type, span, "Comparison operands must have compatible types")?;
                Ok(Type::new(TypeKind::Primitive(PrimitiveType::Bool), span))
            }

            BinaryOp::And | BinaryOp::Or => {
                // Logical operations require boolean types
                self.require_boolean(&left_type, left.span)?;
                self.require_boolean(&right_type, right.span)?;
                Ok(Type::new(TypeKind::Primitive(PrimitiveType::Bool), span))
            }

            _ => {
                Err(TlError::type_error(
                    self.source.clone(),
                    span,
                    format!("Binary operator {:?} not yet implemented", op),
                ))
            }
        }
    }

    /// Type check a unary expression.
    fn check_unary_expr(&mut self, op: &UnaryOp, expr: &mut Expr, span: SourceSpan) -> Result<Type> {
        let expr_type = self.check_expr(expr)?;

        match op {
            UnaryOp::Neg => {
                self.require_numeric(&expr_type, expr.span)?;
                Ok(expr_type)
            }

            UnaryOp::Not => {
                self.require_boolean(&expr_type, expr.span)?;
                Ok(expr_type)
            }

            UnaryOp::BitNot => {
                self.require_integer(&expr_type, expr.span)?;
                Ok(expr_type)
            }
        }
    }

    /// Type check a function call.
    fn check_call_expr(&mut self, callee: &mut Expr, args: &mut [Expr], span: SourceSpan) -> Result<Type> {
        // For now, assume callee is a simple function name
        if let ExprKind::Variable { path } = &callee.kind {
            if path.len() == 1 {
                let func_name = &path[0];
                if let Some(signature) = self.functions.get(func_name).cloned() {
                    // Check argument count
                    if args.len() != signature.params.len() {
                        return Err(TlError::type_error(
                            self.source.clone(),
                            span,
                            format!("Function {} expects {} arguments, got {}",
                                    func_name, signature.params.len(), args.len()),
                        ));
                    }

                    // Check argument types
                    for (i, (arg, expected_type)) in args.iter_mut().zip(signature.params.iter()).enumerate() {
                        let arg_type = self.check_expr(arg)?;
                        self.require_compatible(&arg_type, expected_type, arg.span,
                                                &format!("Argument {} has wrong type", i + 1))?;
                    }

                    Ok(signature.return_type)
                } else {
                    Err(TlError::type_error(
                        self.source.clone(),
                        span,
                        format!("Undefined function: {}", func_name),
                    ))
                }
            } else {
                Err(TlError::type_error(
                    self.source.clone(),
                    span,
                    "Module function calls not yet implemented".to_string(),
                ))
            }
        } else {
            Err(TlError::type_error(
                self.source.clone(),
                span,
                "Function calls on expressions not yet implemented".to_string(),
            ))
        }
    }

    /// Type check an if expression.
    fn check_if_expr(&mut self, condition: &mut Expr, then_branch: &mut Expr,
                     else_branch: &mut Option<Box<Expr>>, span: SourceSpan) -> Result<Type> {
        // Condition must be boolean
        let cond_type = self.check_expr(condition)?;
        self.require_boolean(&cond_type, condition.span)?;

        // Check then branch
        let then_type = self.check_expr(then_branch)?;

        // Check else branch if present
        if let Some(else_expr) = else_branch {
            let else_type = self.check_expr(else_expr)?;
            self.require_compatible(&then_type, &else_type, span,
                                    "If branches must have compatible types")?;
            Ok(then_type)
        } else {
            // If without else returns unit type
            Ok(Type::new(TypeKind::Primitive(PrimitiveType::Unit), span))
        }
    }

    /// Type check a block expression.
    fn check_block_expr(&mut self, block: &mut shared::Block) -> Result<Type> {
        self.push_scope();

        // Check all statements
        for stmt in &mut block.statements {
            self.check_stmt(stmt)?;
        }

        // Check final expression
        let block_type = if let Some(expr) = &mut block.expr {
            self.check_expr(expr)?
        } else {
            Type::new(TypeKind::Primitive(PrimitiveType::Unit), block.span)
        };

        self.pop_scope();
        Ok(block_type)
    }

    /// Type check an assignment expression.
    fn check_assign_expr(&mut self, target: &mut Expr, value: &mut Expr, span: SourceSpan) -> Result<Type> {
        let value_type = self.check_expr(value)?;

        // For now, only support simple variable assignment
        if let ExprKind::Variable { path } = &target.kind {
            if path.len() == 1 {
                let var_name = &path[0];
                if let Some(target_type) = self.variables.get(var_name) {
                    self.require_compatible(&value_type, target_type, span,
                                            "Assignment value type doesn't match variable type")?;
                } else {
                    return Err(TlError::type_error(
                        self.source.clone(),
                        span,
                        format!("Undefined variable in assignment: {}", var_name),
                    ));
                }
            }
        }

        Ok(Type::new(TypeKind::Primitive(PrimitiveType::Unit), span))
    }

    /// Type check a statement.
    fn check_stmt(&mut self, stmt: &mut Stmt) -> Result<()> {
        match &mut stmt.kind {
            StmtKind::Expr(expr) => {
                self.check_expr(expr)?;
            }

            StmtKind::Let { pattern, ty, initializer, .. } => {
                if let PatternKind::Ident(name) = &pattern.kind {
                    let var_type = if let Some(declared_type) = ty {
                        declared_type.clone()
                    } else if let Some(init_expr) = initializer {
                        self.check_expr(init_expr)?
                    } else {
                        return Err(TlError::type_error(
                            self.source.clone(),
                            stmt.span,
                            "Let binding must have either type annotation or initializer".to_string(),
                        ));
                    };

                    // If both type and initializer are present, check compatibility
                    if let (Some(declared_type), Some(init_expr)) = (ty, initializer) {
                        let init_type = self.check_expr(init_expr)?;
                        self.require_compatible(&init_type, declared_type, init_expr.span,
                                                "Initializer type doesn't match declared type")?;
                    }

                    self.variables.insert(name.clone(), var_type);
                }
            }

            _ => {
                return Err(TlError::type_error(
                    self.source.clone(),
                    stmt.span,
                    "Statement type checking not implemented".to_string(),
                ));
            }
        }

        Ok(())
    }

    // Type checking helper methods

    fn require_compatible(&mut self, actual: &Type, expected: &Type, span: SourceSpan, message: &str) -> Result<()> {
        if !self.types_compatible(actual, expected) {
            Err(TlError::type_error(
                self.source.clone(),
                span,
                format!("{}: expected {:?}, found {:?}", message, expected.kind, actual.kind),
            ))
        } else {
            Ok(())
        }
    }

    fn require_numeric(&self, ty: &Type, span: SourceSpan) -> Result<()> {
        match &ty.kind {
            TypeKind::Primitive(prim) => match prim {
                PrimitiveType::I8 | PrimitiveType::I16 | PrimitiveType::I32 | PrimitiveType::I64 | PrimitiveType::I128 | PrimitiveType::ISize |
                PrimitiveType::U8 | PrimitiveType::U16 | PrimitiveType::U32 | PrimitiveType::U64 | PrimitiveType::U128 | PrimitiveType::USize |
                PrimitiveType::F32 | PrimitiveType::F64 => Ok(()),
                _ => Err(TlError::type_error(
                    self.source.clone(),
                    span,
                    format!("Expected numeric type, found {:?}", ty.kind),
                )),
            },
            _ => Err(TlError::type_error(
                self.source.clone(),
                span,
                format!("Expected numeric type, found {:?}", ty.kind),
            )),
        }
    }

    fn require_boolean(&self, ty: &Type, span: SourceSpan) -> Result<()> {
        match &ty.kind {
            TypeKind::Primitive(PrimitiveType::Bool) => Ok(()),
            _ => Err(TlError::type_error(
                self.source.clone(),
                span,
                format!("Expected boolean type, found {:?}", ty.kind),
            )),
        }
    }

    fn require_integer(&self, ty: &Type, span: SourceSpan) -> Result<()> {
        match &ty.kind {
            TypeKind::Primitive(prim) => match prim {
                PrimitiveType::I8 | PrimitiveType::I16 | PrimitiveType::I32 | PrimitiveType::I64 | PrimitiveType::I128 | PrimitiveType::ISize |
                PrimitiveType::U8 | PrimitiveType::U16 | PrimitiveType::U32 | PrimitiveType::U64 | PrimitiveType::U128 | PrimitiveType::USize => Ok(()),
                _ => Err(TlError::type_error(
                    self.source.clone(),
                    span,
                    format!("Expected integer type, found {:?}", ty.kind),
                )),
            },
            _ => Err(TlError::type_error(
                self.source.clone(),
                span,
                format!("Expected integer type, found {:?}", ty.kind),
            )),
        }
    }

    fn types_compatible(&self, a: &Type, b: &Type) -> bool {
        // For now, require exact type equality
        // TODO: Implement proper type compatibility rules (subtyping, coercion, etc.)
        a.kind == b.kind
    }

    fn push_scope(&mut self) {
        // TODO: Implement proper scope stack
        // For now, we'll use a simple approach
    }

    fn pop_scope(&mut self) {
        // TODO: Implement proper scope stack
    }

    fn solve_constraints(&mut self) -> Result<()> {
        // TODO: Implement constraint solver for type inference
        Ok(())
    }

    fn add_builtin_functions(&mut self) {
        // Add built-in functions like print
        self.functions.insert("print".to_string(), FunctionSignature {
            params: vec![Type::new(TypeKind::Primitive(PrimitiveType::Str), SourceSpan::new(0.into(), 0))],
            return_type: Type::new(TypeKind::Primitive(PrimitiveType::Unit), SourceSpan::new(0.into(), 0)),
            safety_level: shared::SafetyLevel::Safe,
        });
    }
}
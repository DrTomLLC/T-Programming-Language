// compiler/src/types/inference.rs
//! Type inference engine for T-Lang.
//!
//! Implements Hindley-Milner style type inference with extensions for safety analysis.
//! Designed to handle complex type relationships while maintaining safety guarantees.

use shared::{Type, TypeKind, PrimitiveType, Result, TlError};
use miette::SourceSpan;
use std::collections::{HashMap, HashSet};

/// Type inference context and engine.
pub struct TypeInferer {
    /// Type variable counter for generating fresh variables
    next_var_id: u32,
    /// Current type variable assignments
    substitutions: HashMap<TypeVariable, Type>,
    /// Type constraints to be solved
    constraints: Vec<TypeConstraint>,
    /// Source code for error reporting
    source: String,
}

/// A type variable used during inference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeVariable(u32);

/// Type constraint for the constraint solver.
#[derive(Debug, Clone)]
pub struct TypeConstraint {
    /// Left side of the constraint
    pub left: Type,
    /// Right side of the constraint
    pub right: Type,
    /// Source location where constraint was generated
    pub span: SourceSpan,
    /// Reason for this constraint (for error messages)
    pub reason: ConstraintReason,
}

/// Reasons why type constraints are generated.
#[derive(Debug, Clone)]
pub enum ConstraintReason {
    /// Expression must have a specific type
    ExpressionType(String),
    /// Function call argument type checking
    ArgumentType { function: String, arg_index: usize },
    /// Function return type checking
    ReturnType(String),
    /// Assignment compatibility
    Assignment,
    /// Binary operation type checking
    BinaryOperation(shared::BinaryOp),
    /// Unary operation type checking
    UnaryOperation(shared::UnaryOp),
    /// Pattern matching
    PatternMatch,
    /// Array element type consistency
    ArrayElement,
    /// Struct field type checking
    StructField(String),
    /// Generic parameter instantiation
    GenericInstantiation,
}

/// Inference context for tracking local type information.
#[derive(Debug, Clone)]
pub struct InferenceContext {
    /// Variable types in current scope
    variables: HashMap<String, Type>,
    /// Function signatures
    functions: HashMap<String, FunctionType>,
    /// Expected return type for current function
    expected_return: Option<Type>,
}

/// Function type information for inference.
#[derive(Debug, Clone)]
pub struct FunctionType {
    /// Parameter types
    pub params: Vec<Type>,
    /// Return type
    pub return_type: Type,
    /// Generic type parameters
    pub generics: Vec<TypeVariable>,
}

impl TypeInferer {
    /// Create a new type inferer.
    pub fn new(source: String) -> Self {
        Self {
            next_var_id: 0,
            substitutions: HashMap::new(),
            constraints: Vec::new(),
            source,
        }
    }

    /// Generate a fresh type variable.
    pub fn fresh_var(&mut self) -> TypeVariable {
        let var = TypeVariable(self.next_var_id);
        self.next_var_id += 1;
        var
    }

    /// Create a type from a type variable.
    pub fn var_type(&self, var: TypeVariable, span: SourceSpan) -> Type {
        Type::new(TypeKind::Unknown(var.0), span)
    }

    /// Add a type constraint.
    pub fn add_constraint(&mut self, left: Type, right: Type, span: SourceSpan, reason: ConstraintReason) {
        self.constraints.push(TypeConstraint {
            left,
            right,
            span,
            reason,
        });
    }

    /// Infer the type of an expression and generate constraints.
    pub fn infer_expr(&mut self, expr: &mut shared::Expr, context: &mut InferenceContext) -> Result<Type> {
        match &mut expr.kind {
            shared::ExprKind::Literal(literal) => {
                let inferred_type = self.infer_literal(literal, expr.span)?;
                expr.ty = Some(inferred_type.clone());
                Ok(inferred_type)
            }

            shared::ExprKind::Variable { path } => {
                let inferred_type = self.infer_variable(path, expr.span, context)?;
                expr.ty = Some(inferred_type.clone());
                Ok(inferred_type)
            }

            shared::ExprKind::Binary { left, op, right } => {
                let inferred_type = self.infer_binary(left, op, right, expr.span, context)?;
                expr.ty = Some(inferred_type.clone());
                Ok(inferred_type)
            }

            shared::ExprKind::Unary { op, expr: inner } => {
                let inferred_type = self.infer_unary(op, inner, expr.span, context)?;
                expr.ty = Some(inferred_type.clone());
                Ok(inferred_type)
            }

            shared::ExprKind::Call { callee, args, .. } => {
                let inferred_type = self.infer_call(callee, args, expr.span, context)?;
                expr.ty = Some(inferred_type.clone());
                Ok(inferred_type)
            }

            shared::ExprKind::If { condition, then_branch, else_branch } => {
                let inferred_type = self.infer_if(condition, then_branch, else_branch, expr.span, context)?;
                expr.ty = Some(inferred_type.clone());
                Ok(inferred_type)
            }

            shared::ExprKind::Block(block) => {
                let inferred_type = self.infer_block(block, context)?;
                expr.ty = Some(inferred_type.clone());
                Ok(inferred_type)
            }

            shared::ExprKind::Array { elements, repeat } => {
                let inferred_type = self.infer_array(elements, repeat.as_deref_mut(), expr.span, context)?;
                expr.ty = Some(inferred_type.clone());
                Ok(inferred_type)
            }

            shared::ExprKind::Tuple(elements) => {
                let inferred_type = self.infer_tuple(elements, expr.span, context)?;
                expr.ty = Some(inferred_type.clone());
                Ok(inferred_type)
            }

            _ => {
                // For unsupported expressions, create a fresh type variable
                let var = self.fresh_var();
                let inferred_type = self.var_type(var, expr.span);
                expr.ty = Some(inferred_type.clone());
                Ok(inferred_type)
            }
        }
    }

    /// Infer the type of a literal.
    fn infer_literal(&mut self, literal: &shared::Literal, span: SourceSpan) -> Result<Type> {
        let type_kind = match literal {
            shared::Literal::Integer(_) => {
                // Generate a fresh type variable for integer literals
                // This allows them to be coerced to any integer type
                let var = self.fresh_var();
                return Ok(self.var_type(var, span));
            }
            shared::Literal::Float(_) => {
                // Generate a fresh type variable for float literals
                let var = self.fresh_var();
                return Ok(self.var_type(var, span));
            }
            shared::Literal::String(_) => TypeKind::Primitive(PrimitiveType::Str),
            shared::Literal::Char(_) => TypeKind::Primitive(PrimitiveType::Char),
            shared::Literal::Bool(_) => TypeKind::Primitive(PrimitiveType::Bool),
            shared::Literal::Unit => TypeKind::Primitive(PrimitiveType::Unit),
        };

        Ok(Type::new(type_kind, span))
    }

    /// Infer the type of a variable reference.
    fn infer_variable(&mut self, path: &[String], span: SourceSpan, context: &InferenceContext) -> Result<Type> {
        if path.len() == 1 {
            let name = &path[0];
            if let Some(var_type) = context.variables.get(name) {
                Ok(var_type.clone())
            } else {
                Err(TlError::type_error(
                    self.source.clone(),
                    span,
                    format!("Undefined variable: {}", name),
                ))
            }
        } else {
            // Module path - not yet implemented
            let var = self.fresh_var();
            Ok(self.var_type(var, span))
        }
    }

    /// Infer the type of a binary expression.
    fn infer_binary(&mut self, left: &mut shared::Expr, op: &shared::BinaryOp,
                    right: &mut shared::Expr, span: SourceSpan, context: &mut InferenceContext) -> Result<Type> {
        let left_type = self.infer_expr(left, context)?;
        let right_type = self.infer_expr(right, context)?;

        match op {
            shared::BinaryOp::Add | shared::BinaryOp::Sub | shared::BinaryOp::Mul |
            shared::BinaryOp::Div | shared::BinaryOp::Mod => {
                // Arithmetic operations: both operands must be the same numeric type
                self.add_constraint(
                    left_type.clone(),
                    right_type.clone(),
                    span,
                    ConstraintReason::BinaryOperation(*op),
                );

                // Result type is the same as operand types
                Ok(left_type)
            }

            shared::BinaryOp::Eq | shared::BinaryOp::Ne => {
                // Equality: operands must be the same type, result is bool
                self.add_constraint(
                    left_type,
                    right_type,
                    span,
                    ConstraintReason::BinaryOperation(*op),
                );

                Ok(Type::new(TypeKind::Primitive(PrimitiveType::Bool), span))
            }

            shared::BinaryOp::Lt | shared::BinaryOp::Le | shared::BinaryOp::Gt | shared::BinaryOp::Ge => {
                // Comparison: operands must be the same numeric type, result is bool
                self.add_constraint(
                    left_type,
                    right_type,
                    span,
                    ConstraintReason::BinaryOperation(*op),
                );

                Ok(Type::new(TypeKind::Primitive(PrimitiveType::Bool), span))
            }

            shared::BinaryOp::And | shared::BinaryOp::Or => {
                // Logical operations: both operands must be bool, result is bool
                let bool_type = Type::new(TypeKind::Primitive(PrimitiveType::Bool), span);

                self.add_constraint(
                    left_type,
                    bool_type.clone(),
                    span,
                    ConstraintReason::BinaryOperation(*op),
                );

                self.add_constraint(
                    right_type,
                    bool_type.clone(),
                    span,
                    ConstraintReason::BinaryOperation(*op),
                );

                Ok(bool_type)
            }

            _ => {
                // For other operations, create a fresh type variable
                let var = self.fresh_var();
                Ok(self.var_type(var, span))
            }
        }
    }

    /// Infer the type of a unary expression.
    fn infer_unary(&mut self, op: &shared::UnaryOp, expr: &mut shared::Expr,
                   span: SourceSpan, context: &mut InferenceContext) -> Result<Type> {
        let expr_type = self.infer_expr(expr, context)?;

        match op {
            shared::UnaryOp::Neg => {
                // Negation: operand must be numeric, result is same type
                Ok(expr_type)
            }

            shared::UnaryOp::Not => {
                // Logical not: operand must be bool, result is bool
                let bool_type = Type::new(TypeKind::Primitive(PrimitiveType::Bool), span);

                self.add_constraint(
                    expr_type,
                    bool_type.clone(),
                    span,
                    ConstraintReason::UnaryOperation(*op),
                );

                Ok(bool_type)
            }

            shared::UnaryOp::BitNot => {
                // Bitwise not: operand must be integer, result is same type
                Ok(expr_type)
            }
        }
    }

    /// Infer the type of a function call.
    fn infer_call(&mut self, callee: &mut shared::Expr, args: &mut [shared::Expr],
                  span: SourceSpan, context: &mut InferenceContext) -> Result<Type> {
        let callee_type = self.infer_expr(callee, context)?;

        // Infer argument types
        let arg_types: Result<Vec<Type>> = args.iter_mut()
            .map(|arg| self.infer_expr(arg, context))
            .collect();
        let arg_types = arg_types?;

        // Create fresh type variable for return type
        let return_var = self.fresh_var();
        let return_type = self.var_type(return_var, span);

        // Create function type constraint
        let func_type = Type::new(
            TypeKind::Function {
                params: arg_types,
                return_type: Box::new(return_type.clone()),
                safety: shared::SafetyLevel::Safe,
            },
            span,
        );

        self.add_constraint(
            callee_type,
            func_type,
            span,
            ConstraintReason::ExpressionType("function call".to_string()),
        );

        Ok(return_type)
    }

    /// Infer the type of an if expression.
    fn infer_if(&mut self, condition: &mut shared::Expr, then_branch: &mut shared::Expr,
                else_branch: &mut Option<Box<shared::Expr>>, span: SourceSpan,
                context: &mut InferenceContext) -> Result<Type> {
        // Condition must be bool
        let cond_type = self.infer_expr(condition, context)?;
        let bool_type = Type::new(TypeKind::Primitive(PrimitiveType::Bool), span);

        self.add_constraint(
            cond_type,
            bool_type,
            condition.span,
            ConstraintReason::ExpressionType("if condition".to_string()),
        );

        // Infer then branch type
        let then_type = self.infer_expr(then_branch, context)?;

        // Infer else branch type if present
        if let Some(else_expr) = else_branch {
            let else_type = self.infer_expr(else_expr, context)?;

            // Both branches must have the same type
            self.add_constraint(
                then_type.clone(),
                else_type,
                span,
                ConstraintReason::ExpressionType("if branches".to_string()),
            );

            Ok(then_type)
        } else {
            // If without else returns unit type
            Ok(Type::new(TypeKind::Primitive(PrimitiveType::Unit), span))
        }
    }

    /// Infer the type of a block expression.
    fn infer_block(&mut self, block: &mut shared::Block, context: &mut InferenceContext) -> Result<Type> {
        // Create new scope
        let saved_vars = context.variables.clone();

        // Process statements
        for stmt in &mut block.statements {
            self.infer_stmt(stmt, context)?;
        }

        // Process final expression
        let block_type = if let Some(expr) = &mut block.expr {
            self.infer_expr(expr, context)?
        } else {
            Type::new(TypeKind::Primitive(PrimitiveType::Unit), block.span)
        };

        // Restore scope
        context.variables = saved_vars;

        Ok(block_type)
    }

    /// Infer the type of an array expression.
    fn infer_array(&mut self, elements: &mut [shared::Expr], repeat: Option<&mut shared::Expr>,
                   span: SourceSpan, context: &mut InferenceContext) -> Result<Type> {
        if let Some(repeat_expr) = repeat {
            // Array with repeat syntax: [expr; count]
            if elements.len() != 1 {
                return Err(TlError::type_error(
                    self.source.clone(),
                    span,
                    "Array repeat syntax requires exactly one element".to_string(),
                ));
            }

            let elem_type = self.infer_expr(&mut elements[0], context)?;
            let count_type = self.infer_expr(repeat_expr, context)?;

            // Count must be integer
            let int_type = Type::new(TypeKind::Primitive(PrimitiveType::USize), span);
            self.add_constraint(
                count_type,
                int_type,
                repeat_expr.span,
                ConstraintReason::ExpressionType("array size".to_string()),
            );

            Ok(Type::new(
                TypeKind::Array {
                    element: Box::new(elem_type),
                    size: shared::types::ArraySize::Inferred,
                },
                span,
            ))
        } else {
            // Regular array: [elem1, elem2, ...]
            if elements.is_empty() {
                // Empty array - create fresh type variable for element type
                let elem_var = self.fresh_var();
                let elem_type = self.var_type(elem_var, span);

                return Ok(Type::new(
                    TypeKind::Array {
                        element: Box::new(elem_type),
                        size: shared::types::ArraySize::Literal(0),
                    },
                    span,
                ));
            }

            // Infer type of first element
            let first_type = self.infer_expr(&mut elements[0], context)?;

            // All elements must have the same type
            for elem in &mut elements[1..] {
                let elem_type = self.infer_expr(elem, context)?;
                self.add_constraint(
                    elem_type,
                    first_type.clone(),
                    elem.span,
                    ConstraintReason::ArrayElement,
                );
            }

            Ok(Type::new(
                TypeKind::Array {
                    element: Box::new(first_type),
                    size: shared::types::ArraySize::Literal(elements.len() as u64),
                },
                span,
            ))
        }
    }

    /// Infer the type of a tuple expression.
    fn infer_tuple(&mut self, elements: &mut [shared::Expr], span: SourceSpan,
                   context: &mut InferenceContext) -> Result<Type> {
        let elem_types: Result<Vec<Type>> = elements.iter_mut()
            .map(|elem| self.infer_expr(elem, context))
            .collect();

        Ok(Type::new(TypeKind::Tuple(elem_types?), span))
    }

    /// Infer types for a statement.
    fn infer_stmt(&mut self, stmt: &mut shared::Stmt, context: &mut InferenceContext) -> Result<()> {
        match &mut stmt.kind {
            shared::StmtKind::Expr(expr) => {
                self.infer_expr(expr, context)?;
            }

            shared::StmtKind::Let { pattern, ty, initializer, .. } => {
                if let shared::PatternKind::Ident(name) = &pattern.kind {
                    let var_type = if let Some(declared_type) = ty {
                        // Type is explicitly declared
                        if let Some(init_expr) = initializer {
                            let init_type = self.infer_expr(init_expr, context)?;

                            // Add constraint that initializer matches declared type
                            self.add_constraint(
                                init_type,
                                declared_type.clone(),
                                init_expr.span,
                                ConstraintReason::Assignment,
                            );
                        }

                        declared_type.clone()
                    } else if let Some(init_expr) = initializer {
                        // Infer type from initializer
                        self.infer_expr(init_expr, context)?
                    } else {
                        // No type annotation or initializer - create fresh variable
                        let var = self.fresh_var();
                        self.var_type(var, stmt.span)
                    };

                    // Add variable to context
                    context.variables.insert(name.clone(), var_type);
                }
            }

            _ => {
                // Other statement types - not yet implemented
            }
        }

        Ok(())
    }

    /// Solve all accumulated type constraints.
    pub fn solve_constraints(&mut self) -> Result<()> {
        // Simple constraint solver using unification
        // This is a basic implementation - a production system would be more sophisticated

        let mut changed = true;
        while changed {
            changed = false;

            for constraint in &self.constraints.clone() {
                if self.unify(&constraint.left, &constraint.right, constraint.span)? {
                    changed = true;
                }
            }
        }

        Ok(())
    }

    /// Apply substitutions to a type.
    pub fn apply_substitutions(&self, ty: &Type) -> Type {
        match &ty.kind {
            TypeKind::Unknown(var_id) => {
                let var = TypeVariable(*var_id);
                if let Some(substituted) = self.substitutions.get(&var) {
                    self.apply_substitutions(substituted)
                } else {
                    ty.clone()
                }
            }

            TypeKind::Reference { target, lifetime, mutable } => {
                Type::new(
                    TypeKind::Reference {
                        target: Box::new(self.apply_substitutions(target)),
                        lifetime: lifetime.clone(),
                        mutable: *mutable,
                    },
                    ty.span,
                )
            }

            TypeKind::Array { element, size } => {
                Type::new(
                    TypeKind::Array {
                        element: Box::new(self.apply_substitutions(element)),
                        size: size.clone(),
                    },
                    ty.span,
                )
            }

            TypeKind::Tuple(elements) => {
                Type::new(
                    TypeKind::Tuple(
                        elements.iter().map(|t| self.apply_substitutions(t)).collect()
                    ),
                    ty.span,
                )
            }

            TypeKind::Function { params, return_type, safety } => {
                Type::new(
                    TypeKind::Function {
                        params: params.iter().map(|t| self.apply_substitutions(t)).collect(),
                        return_type: Box::new(self.apply_substitutions(return_type)),
                        safety: *safety,
                    },
                    ty.span,
                )
            }

            _ => ty.clone(),
        }
    }

    /// Unify two types.
    fn unify(&mut self, left: &Type, right: &Type, span: SourceSpan) -> Result<bool> {
        let left = self.apply_substitutions(left);
        let right = self.apply_substitutions(right);

        match (&left.kind, &right.kind) {
            // Same types unify
            (a, b) if a == b => Ok(false),

            // Type variables unify with anything
            (TypeKind::Unknown(var_id), _) => {
                let var = TypeVariable(*var_id);
                if self.occurs_check(var, &right) {
                    Err(TlError::type_error(
                        self.source.clone(),
                        span,
                        "Infinite type detected".to_string(),
                    ))
                } else {
                    self.substitutions.insert(var, right);
                    Ok(true)
                }
            }

            (_, TypeKind::Unknown(var_id)) => {
                let var = TypeVariable(*var_id);
                if self.occurs_check(var, &left) {
                    Err(TlError::type_error(
                        self.source.clone(),
                        span,
                        "Infinite type detected".to_string(),
                    ))
                } else {
                    self.substitutions.insert(var, left);
                    Ok(true)
                }
            }

            // Compound types
            (TypeKind::Array { element: e1, size: s1 }, TypeKind::Array { element: e2, size: s2 }) => {
                if s1 != s2 {
                    return Err(TlError::type_error(
                        self.source.clone(),
                        span,
                        "Array sizes don't match".to_string(),
                    ));
                }
                self.unify(e1, e2, span)
            }

            (TypeKind::Tuple(elems1), TypeKind::Tuple(elems2)) => {
                if elems1.len() != elems2.len() {
                    return Err(TlError::type_error(
                        self.source.clone(),
                        span,
                        "Tuple sizes don't match".to_string(),
                    ));
                }

                let mut changed = false;
                for (e1, e2) in elems1.iter().zip(elems2.iter()) {
                    if self.unify(e1, e2, span)? {
                        changed = true;
                    }
                }
                Ok(changed)
            }

            // Types don't unify
            _ => Err(TlError::type_error(
                self.source.clone(),
                span,
                format!("Cannot unify types: {:?} and {:?}", left.kind, right.kind),
            )),
        }
    }

    /// Check if a type variable occurs in a type (prevents infinite types).
    fn occurs_check(&self, var: TypeVariable, ty: &Type) -> bool {
        match &ty.kind {
            TypeKind::Unknown(var_id) => TypeVariable(*var_id) == var,
            TypeKind::Reference { target, .. } => self.occurs_check(var, target),
            TypeKind::Array { element, .. } => self.occurs_check(var, element),
            TypeKind::Tuple(elements) => elements.iter().any(|t| self.occurs_check(var, t)),
            TypeKind::Function { params, return_type, .. } => {
                params.iter().any(|t| self.occurs_check(var, t)) ||
                    self.occurs_check(var, return_type)
            }
            _ => false,
        }
    }
}

impl InferenceContext {
    /// Create a new inference context.
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            expected_return: None,
        }
    }

    /// Add a variable to the context.
    pub fn add_variable(&mut self, name: String, ty: Type) {
        self.variables.insert(name, ty);
    }

    /// Add a function to the context.
    pub fn add_function(&mut self, name: String, func_type: FunctionType) {
        self.functions.insert(name, func_type);
    }

    /// Set the expected return type for the current function.
    pub fn set_expected_return(&mut self, ty: Type) {
        self.expected_return = Some(ty);
    }
}

impl Default for InferenceContext {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ConstraintReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConstraintReason::ExpressionType(desc) => write!(f, "{}", desc),
            ConstraintReason::ArgumentType { function, arg_index } => {
                write!(f, "argument {} to function {}", arg_index + 1, function)
            }
            ConstraintReason::ReturnType(func) => write!(f, "return type of {}", func),
            ConstraintReason::Assignment => write!(f, "assignment"),
            ConstraintReason::BinaryOperation(op) => write!(f, "binary operation {:?}", op),
            ConstraintReason::UnaryOperation(op) => write!(f, "unary operation {:?}", op),
            ConstraintReason::PatternMatch => write!(f, "pattern match"),
            ConstraintReason::ArrayElement => write!(f, "array element"),
            ConstraintReason::StructField(field) => write!(f, "struct field {}", field),
            ConstraintReason::GenericInstantiation => write!(f, "generic instantiation"),
        }
    }
}
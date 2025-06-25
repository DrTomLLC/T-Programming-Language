// shared/src/ast/mod.rs
use miette::SourceSpan;
use serde::{Deserialize, Serialize};

// Re-export types from parent module
pub use crate::{
    Span, Type, TypeKind, PrimitiveType, Expr, ExprKind, Pattern, PatternKind,
    Stmt, StmtKind, Item, ItemKind, Program, Block, BinaryOp, UnaryOp, Literal,
    Visibility, SafetyLevel, FnParam, StructField, Attribute, GenericParam,
    StructFields, EnumVariant, EnumVariantData, MatchArm, ClosureParam
};

// Additional sub-modules
pub mod expr;
pub mod types;
pub mod patterns;

pub trait HasSpan {
    fn span(&self) -> Span;
}

impl HasSpan for Expr {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for Stmt {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for Item {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for Type {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for Pattern {
    fn span(&self) -> Span {
        self.span
    }
}

/// Helper functions for creating AST nodes.
impl Item {
    pub fn function(
        name: String,
        params: Vec<FnParam>,
        return_type: Option<Type>,
        body: Expr,
        span: Span,
    ) -> Self {
        Self {
            kind: ItemKind::Function {
                name,
                generics: Vec::new(),
                params,
                return_type,
                body,
                safety: SafetyLevel::Safe,
            },
            attrs: Vec::new(),
            vis: Visibility::Private,
            span,
        }
    }

    pub fn struct_def(name: String, fields: Vec<StructField>, span: Span) -> Self {
        Self {
            kind: ItemKind::Struct {
                name,
                generics: Vec::new(),
                fields: StructFields::Named(fields),
            },
            attrs: Vec::new(),
            vis: Visibility::Private,
            span,
        }
    }

    pub fn use_item(path: Vec<String>, span: Span) -> Self {
        Self {
            kind: ItemKind::Use {
                path,
                alias: None,
                glob: false,
            },
            attrs: Vec::new(),
            vis: Visibility::Private,
            span,
        }
    }
}

impl StructField {
    pub fn new(name: String, ty: Type, span: Span) -> Self {
        Self {
            name,
            ty,
            vis: Visibility::Private,
            attrs: Vec::new(),
            span,
        }
    }
}

impl FnParam {
    pub fn new(name: String, ty: Type, span: Span) -> Self {
        Self {
            pattern: Pattern::identifier(name, span),
            ty,
            default: None,
            attrs: Vec::new(),
            span,
        }
    }
}

impl Attribute {
    pub fn new(path: Vec<String>, span: Span) -> Self {
        Self {
            path,
            args: Vec::new(),
            span,
        }
    }

    pub fn simple(name: &str, span: Span) -> Self {
        Self::new(vec![name.to_string()], span)
    }
}

/// AST visitor pattern for traversing the tree.
pub trait Visitor<T = ()> {
    fn visit_program(&mut self, program: &Program) -> T {
        walk_program(self, program)
    }

    fn visit_item(&mut self, item: &Item) -> T {
        walk_item(self, item)
    }

    fn visit_stmt(&mut self, stmt: &Stmt) -> T {
        walk_stmt(self, stmt)
    }

    fn visit_expr(&mut self, expr: &Expr) -> T {
        walk_expr(self, expr)
    }

    fn visit_type(&mut self, ty: &Type) -> T {
        walk_type(self, ty)
    }

    fn visit_pattern(&mut self, pattern: &Pattern) -> T {
        walk_pattern(self, pattern)
    }
}

/// Default walking implementations.
pub fn walk_program<V: Visitor<T>, T>(visitor: &mut V, program: &Program) -> T {
    let mut results = Vec::new();
    for item in &program.items {
        results.push(visitor.visit_item(item));
    }
    // Return the last result or default
    results.into_iter().last().unwrap_or_else(|| {
        // For unit type T = (), we can create a default
        unsafe { std::mem::zeroed() }
    })
}

pub fn walk_item<V: Visitor<T>, T>(visitor: &mut V, item: &Item) -> T {
    match &item.kind {
        ItemKind::Function { body, .. } => visitor.visit_expr(body),
        ItemKind::Struct { .. } => unsafe { std::mem::zeroed() },
        ItemKind::Enum { .. } => unsafe { std::mem::zeroed() },
        ItemKind::Use { .. } => unsafe { std::mem::zeroed() },
        ItemKind::Const { init, .. } => visitor.visit_expr(init),
        ItemKind::Static { init, .. } => visitor.visit_expr(init),
        ItemKind::Module { items, .. } => {
            let mut results = Vec::new();
            for item in items {
                results.push(visitor.visit_item(item));
            }
            results.into_iter().last().unwrap_or_else(|| unsafe { std::mem::zeroed() })
        }
    }
}

pub fn walk_stmt<V: Visitor<T>, T>(visitor: &mut V, stmt: &Stmt) -> T {
    match &stmt.kind {
        StmtKind::Local { init, .. } => {
            if let Some(init) = init {
                visitor.visit_expr(init)
            } else {
                unsafe { std::mem::zeroed() }
            }
        }
        StmtKind::Item(item) => visitor.visit_item(item),
        StmtKind::Expr(expr) | StmtKind::Semi(expr) => visitor.visit_expr(expr),
    }
}

pub fn walk_expr<V: Visitor<T>, T>(visitor: &mut V, expr: &Expr) -> T {
    match &expr.kind {
        ExprKind::Literal(_) | ExprKind::Identifier(_) | ExprKind::Path(_) => {
            unsafe { std::mem::zeroed() }
        }
        ExprKind::Binary { lhs, rhs, .. } => {
            visitor.visit_expr(lhs);
            visitor.visit_expr(rhs)
        }
        ExprKind::Unary { operand, .. } => visitor.visit_expr(operand),
        ExprKind::Call { func, args, .. } => {
            visitor.visit_expr(func);
            let mut last = unsafe { std::mem::zeroed() };
            for arg in args {
                last = visitor.visit_expr(arg);
            }
            last
        }
        ExprKind::FieldAccess { object, .. } => visitor.visit_expr(object),
        ExprKind::Index { object, index } => {
            visitor.visit_expr(object);
            visitor.visit_expr(index)
        }
        ExprKind::Tuple(exprs) | ExprKind::Array(exprs) => {
            let mut last = unsafe { std::mem::zeroed() };
            for expr in exprs {
                last = visitor.visit_expr(expr);
            }
            last
        }
        ExprKind::Block(block) => {
            let mut last = unsafe { std::mem::zeroed() };
            for stmt in &block.stmts {
                last = visitor.visit_stmt(stmt);
            }
            last
        }
        ExprKind::If { condition, then_branch, else_branch } => {
            visitor.visit_expr(condition);
            visitor.visit_expr(then_branch);
            if let Some(else_branch) = else_branch {
                visitor.visit_expr(else_branch)
            } else {
                unsafe { std::mem::zeroed() }
            }
        }
        ExprKind::While { condition, body } => {
            visitor.visit_expr(condition);
            visitor.visit_expr(body)
        }
        ExprKind::For { iterator, body, .. } => {
            visitor.visit_expr(iterator);
            visitor.visit_expr(body)
        }
        ExprKind::Match { expr, arms } => {
            visitor.visit_expr(expr);
            let mut last = unsafe { std::mem::zeroed() };
            for arm in arms {
                visitor.visit_pattern(&arm.pattern);
                if let Some(guard) = &arm.guard {
                    visitor.visit_expr(guard);
                }
                last = visitor.visit_expr(&arm.body);
            }
            last
        }
        ExprKind::Return(expr) | ExprKind::Break(expr) => {
            if let Some(expr) = expr {
                visitor.visit_expr(expr)
            } else {
                unsafe { std::mem::zeroed() }
            }
        }
        ExprKind::Continue => unsafe { std::mem::zeroed() },
        ExprKind::Closure { body, .. } => visitor.visit_expr(body),
    }
}

pub fn walk_type<V: Visitor<T>, T>(_visitor: &mut V, _ty: &Type) -> T {
    // For now, types don't have nested structures to visit
    unsafe { std::mem::zeroed() }
}

// Fixed function signature - removed ?Sized in wrong position
pub fn walk_pattern<V: Visitor<T>, T>(_visitor: &mut V, _pattern: &Pattern) -> T {
    // For now, patterns don't need complex visiting
    unsafe { std::mem::zeroed() }
}
// shared/src/ast/mod.rs
//! Abstract Syntax Tree definitions for T-Lang.
//! Complete AST system supporting all language features.

use miette::SourceSpan;
use serde::{Deserialize, Serialize};

pub mod expr;
pub mod stmt;
pub mod types;
mod ast;

// Re-export all the AST node types
pub use expr::*;
pub use stmt::*;
pub use types::*;

/// Root program node containing all top-level items.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub items: Vec<Item>,
    pub span: SourceSpan,
}

/// A module containing items.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    pub items: Vec<Item>,
    pub span: SourceSpan,
}

/// Source span information for error reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub len: usize,
}

impl Span {
    pub fn new(start: usize, len: usize) -> Self {
        Self { start, len }
    }

    pub fn dummy() -> Self {
        Self { start: 0, len: 0 }
    }

    pub fn end(&self) -> usize {
        self.start + self.len
    }

    pub fn contains(&self, offset: usize) -> bool {
        offset >= self.start && offset < self.end()
    }

    pub fn overlaps(&self, other: &Span) -> bool {
        self.start < other.end() && other.start < self.end()
    }

    pub fn merge(&self, other: &Span) -> Span {
        let start = self.start.min(other.start);
        let end = self.end().max(other.end());
        Span::new(start, end - start)
    }
}

impl From<SourceSpan> for Span {
    fn from(span: SourceSpan) -> Self {
        Self::new(span.offset(), span.len())
    }
}

impl From<Span> for SourceSpan {
    fn from(span: Span) -> Self {
        SourceSpan::new(span.start.into(), span.len)
    }
}

/// Unique identifier for AST nodes.
/// Used for type inference, dependency analysis, and incremental compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u32);

impl NodeId {
    pub const DUMMY: NodeId = NodeId(u32::MAX);

    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

/// Compilation phase marker for AST nodes.
/// Tracks which compilation phases have been applied to this node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CompilationPhase {
    pub parsed: bool,
    pub name_resolved: bool,
    pub type_checked: bool,
    pub safety_checked: bool,
    pub optimized: bool,
}

impl Default for CompilationPhase {
    fn default() -> Self {
        Self {
            parsed: true,
            name_resolved: false,
            type_checked: false,
            safety_checked: false,
            optimized: false,
        }
    }
}

/// Metadata attached to AST nodes for compilation tracking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub id: NodeId,
    pub phase: CompilationPhase,
    pub source_file: Option<String>,
}

impl Default for NodeMetadata {
    fn default() -> Self {
        Self {
            id: NodeId::DUMMY,
            phase: CompilationPhase::default(),
            source_file: None,
        }
    }
}

/// Top-level items (functions, types, modules, etc.)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub kind: ItemKind,
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub span: SourceSpan,
}

/// All possible item kinds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ItemKind {
    /// Function definition: fn name(params) -> return_type { body }
    Function {
        name: String,
        generics: Vec<GenericParam>,
        params: Vec<FnParam>,
        return_type: Option<Type>,
        body: Expr,
        safety: SafetyLevel,
    },

    /// Struct definition: struct Name { fields }
    Struct {
        name: String,
        generics: Vec<GenericParam>,
        fields: StructFields,
    },

    /// Enum definition: enum Name { variants }
    Enum {
        name: String,
        generics: Vec<GenericParam>,
        variants: Vec<EnumVariant>,
    },

    /// Type alias: type Name = Type;
    TypeAlias {
        name: String,
        generics: Vec<GenericParam>,
        ty: Type,
    },

    /// Trait definition: trait Name { items }
    Trait {
        name: String,
        generics: Vec<GenericParam>,
        supertraits: Vec<Type>,
        items: Vec<TraitItem>,
    },

    /// Implementation: impl [Type for] Type { items }
    Impl {
        generics: Vec<GenericParam>,
        trait_ref: Option<Type>,
        self_ty: Type,
        items: Vec<ImplItem>,
    },

    /// Module: mod name { items }
    Module {
        name: String,
        items: Vec<Item>,
    },

    /// Use declaration: use path;
    Use {
        path: Vec<String>,
        alias: Option<String>,
        glob: bool,
    },

    /// Constant: const NAME: Type = expr;
    Const {
        name: String,
        ty: Type,
        value: Expr,
    },

    /// Static variable: static NAME: Type = expr;
    Static {
        name: String,
        ty: Type,
        value: Expr,
        mutable: bool,
    },

    /// External block: extern "C" { items }
    Extern {
        abi: Option<String>,
        items: Vec<ExternItem>,
    },

    /// Macro definition: macro_rules! name { rules }
    Macro {
        name: String,
        rules: Vec<MacroRule>,
    },
}

/// Struct field definitions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StructFields {
    Named(Vec<StructField>),     // struct S { x: i32, y: i32 }
    Unnamed(Vec<Type>),          // struct S(i32, i32);
    Unit,                        // struct S;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructField {
    pub name: String,
    pub ty: Type,
    pub vis: Visibility,
    pub attrs: Vec<Attribute>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumVariant {
    pub name: String,
    pub fields: StructFields,
    pub discriminant: Option<Expr>,
    pub attrs: Vec<Attribute>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FnParam {
    pub pattern: Pattern,
    pub ty: Type,
    pub default: Option<Expr>,
    pub attrs: Vec<Attribute>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<Type>,
    pub default: Option<Type>,
    pub span: SourceSpan,
}

/// Trait items.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TraitItem {
    Function {
        name: String,
        generics: Vec<GenericParam>,
        params: Vec<FnParam>,
        return_type: Option<Type>,
        body: Option<Expr>,
        safety: SafetyLevel,
    },
    Type {
        name: String,
        bounds: Vec<Type>,
        default: Option<Type>,
    },
    Const {
        name: String,
        ty: Type,
        value: Option<Expr>,
    },
}

/// Implementation items.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImplItem {
    Function {
        name: String,
        generics: Vec<GenericParam>,
        params: Vec<FnParam>,
        return_type: Option<Type>,
        body: Expr,
        safety: SafetyLevel,
        vis: Visibility,
    },
    Type {
        name: String,
        ty: Type,
        vis: Visibility,
    },
    Const {
        name: String,
        ty: Type,
        value: Expr,
        vis: Visibility,
    },
}

/// External items.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExternItem {
    Function {
        name: String,
        params: Vec<FnParam>,
        return_type: Option<Type>,
        variadic: bool,
    },
    Static {
        name: String,
        ty: Type,
        mutable: bool,
    },
}

/// Visibility levels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Visibility {
    Public,                      // pub
    PublicCrate,                 // pub(crate)
    PublicSuper,                 // pub(super)
    PublicIn(Vec<String>),       // pub(in path::to::module)
    Private,                     // (default)
}

/// Attributes like #[derive(Debug)], #[inline].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attribute {
    pub path: Vec<String>,
    pub args: Vec<MacroArg>,
    pub span: SourceSpan,
}

/// Macro arguments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MacroArg {
    Token(crate::token::Token),
    Group {
        delimiter: MacroDelimiter,
        tokens: Vec<MacroArg>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MacroDelimiter {
    Parentheses,
    Brackets,
    Braces,
}

/// Macro rules for macro_rules! definitions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacroRule {
    pub pattern: Vec<MacroArg>,
    pub body: Vec<MacroArg>,
}

/// Safety levels for functions and operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SafetyLevel {
    Safe,
    Unsafe,
}

impl Default for SafetyLevel {
    fn default() -> Self {
        Self::Safe
    }
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Private
    }
}

impl Program {
    /// Create a new empty program.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            span: SourceSpan::new(0.into(), 0),
        }
    }

    /// Add an item to the program.
    pub fn add_item(&mut self, item: Item) {
        self.items.push(item);
    }

    /// Get all functions in the program.
    pub fn functions(&self) -> impl Iterator<Item = &Item> {
        self.items.iter().filter(|item| {
            matches!(item.kind, ItemKind::Function { .. })
        })
    }

    /// Get all modules in the program.
    pub fn modules(&self) -> impl Iterator<Item = &Item> {
        self.items.iter().filter(|item| {
            matches!(item.kind, ItemKind::Module { .. })
        })
    }

    /// Check if the program is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl Module {
    /// Create a new module.
    pub fn new(name: String, span: SourceSpan) -> Self {
        Self {
            name,
            items: Vec::new(),
            span,
        }
    }

    /// Add an item to the module.
    pub fn add_item(&mut self, item: Item) {
        self.items.push(item);
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper trait for getting spans from AST nodes.
pub trait HasSpan {
    fn span(&self) -> SourceSpan;
}

impl HasSpan for Expr {
    fn span(&self) -> SourceSpan {
        self.span
    }
}

impl HasSpan for Stmt {
    fn span(&self) -> SourceSpan {
        self.span
    }
}

impl HasSpan for Item {
    fn span(&self) -> SourceSpan {
        self.span
    }
}

impl HasSpan for Type {
    fn span(&self) -> SourceSpan {
        self.span
    }
}

impl HasSpan for Pattern {
    fn span(&self) -> SourceSpan {
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
        span: SourceSpan,
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

    pub fn struct_def(name: String, fields: Vec<StructField>, span: SourceSpan) -> Self {
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

    pub fn use_item(path: Vec<String>, span: SourceSpan) -> Self {
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
    pub fn new(name: String, ty: Type, span: SourceSpan) -> Self {
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
    pub fn new(name: String, ty: Type, span: SourceSpan) -> Self {
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
    pub fn new(path: Vec<String>, span: SourceSpan) -> Self {
        Self {
            path,
            args: Vec::new(),
            span,
        }
    }

    pub fn simple(name: &str, span: SourceSpan) -> Self {
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

    fn visit_pattern(&mut self, pattern: &Pattern) -> T where Self: std::marker::Sized {
        walk_pattern(self, pattern)
    }
}

/// Default walking implementations.
pub fn walk_program<V: Visitor<T>, T>(_visitor: &mut V, _program: &Program) -> T {
    todo!("Implement AST walking")
}

pub fn walk_item<V: Visitor<T>, T>(_visitor: &mut V, _item: &Item) -> T {
    todo!("Implement AST walking")
}

pub fn walk_stmt<V: Visitor<T>, T>(_visitor: &mut V, _stmt: &Stmt) -> T {
    todo!("Implement AST walking")
}

pub fn walk_expr<V: Visitor<T>, T>(_visitor: &mut V, _expr: &Expr) -> T {
    todo!("Implement AST walking")
}

pub fn walk_type<V: Visitor<T>, T>(_visitor: &mut V, _ty: &Type) -> T {
    todo!("Implement AST walking")
}

pub fn walk_pattern<V: Visitor<T>, T>(_visitor: &mut V, _pattern: &Pattern) -> T {
    todo!("Implement AST walking")
}
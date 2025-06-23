# Parser & AST

This document provides a deep dive into the design and implementation of T‑Lang’s parser and Abstract Syntax Tree (AST).

---

## 1. Grammar Design

### 1.1 Goals

* **Expressiveness**: Capture T‑Lang’s syntax for modules, declarations, expressions, and control flow in a clear, context‑free grammar.
* **Unambiguous**: Minimize shift/reduce and reduce/reduce conflicts to simplify the parser.
* **Extensible**: Support future language features (e.g., macros, generics, patterns) without wholesale grammar rewrites.
* **Maintainable**: Use EBNF notation and modular rule definitions for readability and tool integration.

### 1.2 Lexical Structure

* **Whitespace & Comments**: Ignored by parser. Two forms of comments:

    * Line comments: `// ...` rest of line
    * Block comments: `/* ... */` nesting allowed.
* **Tokens**:

    * **Literals**: integers, floats, strings, booleans
    * **Identifiers**: `[A-Za-z_][A-Za-z0-9_]*`
    * **Operators**: `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `>`, `<=`, `>=`, etc.
    * **Delimiters**: `(`, `)`, `{`, `}`, `[`, `]`, `;`, `,`, `::`, `->`, etc.

### 1.3 High‑Level Grammar (EBNF)

```ebnf
Program       ::= ModuleDecl* EOF
ModuleDecl    ::= "module" Identifier ("::" Identifier)* "{" TopLevelItem* "}"
TopLevelItem  ::= Import | FunctionDecl | StructDecl | EnumDecl | ConstDecl
Import        ::= "import" Path ("as" Identifier)? ";"
FunctionDecl  ::= "fn" Identifier "(" ParamList? ")" ("->" Type)? Block
ParamList     ::= Param ("," Param)*
Param         ::= Identifier ":" Type
StructDecl    ::= "struct" Identifier "{" FieldList? "}" ";"
EnumDecl      ::= "enum" Identifier "{" VariantList? "}" ";"
ConstDecl     ::= "const" Identifier ":" Type "=" Expression ";"
Expression    ::= Assignment
Assignment    ::= LogicalOr ("=" Assignment)?
LogicalOr     ::= LogicalAnd ("||" LogicalAnd)*
LogicalAnd    ::= Equality ("&&" Equality)*
Equality      ::= Comparison (("==" | "!=") Comparison)*
Comparison    ::= Term (("<" | ">" | "<=" | ">=") Term)*
Term          ::= Factor (("+" | "-") Factor)*
Factor        ::= Unary (("*" | "/" | "%") Unary)*
Unary         ::= ("!" | "-") Unary | Primary
Primary       ::= Literal | Identifier | CallExpr | "(" Expression ")"
CallExpr      ::= Identifier "(" ArgList? ")"
ArgList       ::= Expression ("," Expression)*
Block         ::= "{" Statement* "}"
Statement     ::= ExpressionStmt | LetDecl | IfStmt | WhileStmt | ReturnStmt
```

---

## 2. AST Node Definitions

Each grammar production corresponds to one or more AST node types. The AST carries both semantic structure and source-location information.

### 2.1 Position & Span

```rust
/// Represents a location in source code.
pub struct Span {
    pub start: usize,
    pub end: usize,
}
```

Every AST node embeds a `Span` for error reporting.

### 2.2 Top‑Level AST

```rust
pub enum TopLevelItem {
    Import(ImportDecl),
    Function(FuncDecl),
    Struct(StructDecl),
    Enum(EnumDecl),
    Const(ConstDecl),
}
```

### 2.3 Expressions & Statements

```rust
pub enum Expr {
    Literal(Literal),
    Ident(String),
    Binary(Box<Expr>, BinOp, Box<Expr>),
    Unary(UnOp, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    // ... more (Field access, indexing, closures, etc.)
}

pub enum Stmt {
    Expr(Expr),
    Let(String, Type, Option<Expr>),
    If(Expr, Block, Option<Block>),
    While(Expr, Block),
    Return(Option<Expr>),
    // ...
}
```

### 2.4 Declarations

```rust
pub struct FuncDecl {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Option<Type>,
    pub body: Block,
    pub span: Span,
}
// StructDecl, EnumDecl, ConstDecl similarly modeled.
```

---

## 3. Parser Architecture & Error Recovery

### 3.1 Parser Type

* **Recursive‑descent**: clear control over precedence climbing.
* **Stateful**: maintains current token, lookahead, and error list.

### 3.2 Error Handling Strategies

1. **Panic‑mode recovery**:

    * On unexpected token, record a diagnostic, skip tokens until a synchronization point (e.g., `;`, `}`, or newline).
2. **Error productions**:

    * Allow certain common mistakes (e.g., missing semicolon) to consume a token and continue.
3. **Aggregated diagnostics**:

    * Collect all parse errors, report at end, rather than aborting on first.

### 3.3 Synchronization Points

* Semicolon (`;`)
* Closing brace (`}`)
* Top‑level boundaries (`module`, `fn`, `struct`)

---

## 4. Extension Points & Plugin Hooks

To support future language features and plugins, the parser exposes:

* **AST Visitors & Transformers**:

  ```rust
  pub trait ASTVisitor {
      fn visit_expr(&mut self, expr: &Expr) -> Control;
      fn visit_stmt(&mut self, stmt: &Stmt) -> Control;
      // ...
  }
  ```
* **Custom grammar injections**:

    * Pre‑parse hook to inject or rewrite token stream (e.g., macro expansions).
* **Attributes & Annotations**:

    * Attach user‑defined metadata to AST nodes, preserved through semantic analysis and codegen.

---

*(Next: Semantic Analysis & Type System)*

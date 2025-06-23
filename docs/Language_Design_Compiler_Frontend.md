# Language Design & Compiler Frontend

This section specifies how T‑Lang source programs are parsed, analyzed, and transformed into the intermediate representation (TIR). It covers lexical grammar, concrete & abstract syntax, semantic analysis, type checking, TIR generation, plugin extension points, and diagnostics.

---

## 1. Objectives & Scope

* **Purpose**: Define a robust, extensible frontend pipeline that ensures source correctness, rich diagnostics, and a solid foundation for optimization and code generation.
* **Scope**:

    * Lexical analysis & tokenization
    * Grammar specification & parsing
    * AST construction & transformation
    * Semantic analysis (name resolution, type checking)
    * TIR lowering
    * Plugin API integration at each phase
    * Error reporting with precise source spans

---

## 2. Lexical Grammar & Syntax

### 2.1 Lexical Structure

* **Tokens**: identifiers, keywords, literals (numeric, string, char), operators, delimiters, comments.
* **Rules**:

    * Whitespace and comments are skipped.
    * Unicode support in identifiers.
    * Raw string literals and escape sequences.

### 2.2 Grammar (EBNF)

```ebnf
Program      ::= { Item }
Item         ::= Function | Struct | Enum | Import | Export | ModuleDecl | ...
Function     ::= "fn" Identifier "(" [ ParamList ] ")" [ "->" Type ] Block
ParamList    ::= Param { "," Param }
Param        ::= Identifier ":" Type
Block        ::= "{" { Statement } "}"
Statement    ::= LetStmt | ExprStmt | ReturnStmt | IfStmt | LoopStmt | ...
ExprStmt     ::= Expression ";"
Expression   ::= Literal | Identifier | CallExpr | BinaryExpr | ...
CallExpr     ::= Expression "(" [ ArgList ] ")"
BinaryExpr   ::= Expression Operator Expression
```

### 2.3 Macro & Metaprogramming

* **Built‑in macros** (`include!`, `println!`).
* **Custom derive** for code generation (planned future extension).

---

## 3. Abstract Syntax Tree (AST)

### 3.1 AST Data Structures

* `AstNode`: enum of all syntax constructs (Module, Item, Expr, Type, Pattern, etc.).
* Span information attached to each node for error reporting.

### 3.2 AST Construction

* **Parser**: recursive‑descent or table‑driven parser emitting AST nodes.
* **Error recovery**: synchronize at semicolons or braces to continue parsing.

### 3.3 AST Visitors & Transformers

* `ASTVisitor`: read‑only traversal.
* `ASTTransformer`: in‑place or cloned transformation, used for desugaring (e.g. `for` loops to `while`).

---

## 4. Semantic Analysis

### 4.1 Name Resolution

* Build symbol tables per module, respect scopes (block, function, module).
* Support for imports, re‑exports, and name shadowing.
* Error on undefined identifiers or ambiguous references.

### 4.2 Type System & Inference

* **Primitive types**: integers, floats, booleans, characters, strings.
* **Composite types**: arrays, tuples, functions, generics.
* **Type inference**: Hindley‑Milner style local inference for `let` bindings and function return types.
* **Generic constraints**: trait bounds (planned), where‑clauses.
* **Error reporting**: unification failures with explanatory messages.

### 4.3 Borrow & Ownership Checks (Future)

* Outline of planned borrow‑checker for memory safety.
* Integration points in semantic phase.

---

## 5. TIR Generation & Lowering

### 5.1 TIR Overview

* **Structure**: Modules, functions, basic blocks, instructions (SSA‑based).
* **Value types**: primitive, pointer, aggregate.

### 5.2 Lowering Steps

1. **Desugar AST**: expand syntactic sugar.
2. **Control‑flow analysis**: build CFG skeleton.
3. **Emit SSA**: translate expressions & statements into TIR operations.

### 5.3 Optimization Hooks

* Constant folding, dead code elimination, inlining.
* Plugin hook after each pass for custom transforms.

---

## 6. Plugin & Extension API (Frontend)

* Define trait `FrontendPlugin` with methods:

  ```rust
  trait FrontendPlugin {
      fn name(&self) -> &str;
      fn after_parse(&self, ast: &mut AstModule) -> Result<(), Error>;
      fn after_semantic(&self, symbols: &SymbolTable) -> Result<(), Error>;
      fn before_tir(&self, module: &mut TirModule) -> Result<(), Error>;
  }
  ```
* Plugins registered via `PluginRegistry` during compiler bootstrap.

---

## 7. Error Reporting & Diagnostics

* Unified `Diagnostic` type with:

    * Span (file, line, column start/end)
    * Severity (Error, Warning, Note)
    * Message and optional suggestions
* Aggregate diagnostics and present sorted by location.
* Support IDE integrations via JSON emitter.

---

## 8. Testing & Validation

* **Parser tests**: invalid and valid syntax suites.
* **Semantic tests**: name resolution and type inference edge cases.
* **Round‑trip tests**: AST → TIR → back‑translate (future).

---

## 9. Future Extensions

* **Macro system**: hygienic macros.
* **Trait & module augmentation**.
* **Incremental compilation** in frontend.

---

*Next:* **Code Generation & Backend Integration**, covering Cranelift, LLVM, C, Zig, and custom backends.

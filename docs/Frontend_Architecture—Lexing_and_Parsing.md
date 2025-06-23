# Frontend Architecture — Lexing & Parsing

This section defines the design and implementation details of the T‑Lang frontend: turning raw source into an abstract syntax representation and then lowering to TIR.

---

## 1. Lexical Analysis (Lexer)

### 1.1. Token Model

* **Token Types**: identifiers, keywords, literals (integer, float, string, char), operators, punctuation, comments, whitespace.
* **Location Tracking**: each token carries `span: (start_line, start_col, end_line, end_col)` for precise errors.

### 1.2. Design & Implementation

1. **Streaming API**: iterator over `char` input, buffering at most one lookahead.
2. **Unicode & Escapes**: support full Unicode in identifiers and string/char escapes (`\n`, `\u{...}`).
3. **Keyword vs Identifier**: recognize keywords by a lookup table after scanning identifier lexeme.
4. **Comments & Whitespace**:

    * Single-line `//...` and multi-line `/* ... */`, nested allowed in multi-line.
    * Emit comment tokens only under `--dump-tokens`; otherwise skip.

### 1.3. Error Handling

* **Invalid Character**: emit `LexError::InvalidCharacter(span, ch)` and attempt to continue at next codepoint.
* **Unterminated String/Comment**: `LexError::Unterminated(span, kind)` with recovery by skipping to next newline (strings) or closing comment delimiter.

### 1.4. Testing

* Unit tests for each token class, boundary conditions, error cases.
* Fuzz tests on random byte streams to ensure resilience.

---

## 2. Parsing (Parser)

### 2.1. Grammar Specification

* **Grammar Style**: LL(1)/recursive-descent with limited lookahead.
* **Key Productions**:

    * `Program    := Item*`
    * `Item       := fn_decl | struct_decl | enum_decl | import_stmt | ...`
    * `Expr       := binary_expr | unary_expr | primary_expr`
    * `Stmt       := let_stmt | expr_stmt | return_stmt | ...`

### 2.2. Parser Strategy

1. **Recursive-Descent**: handwritten functions `parse_<nonterminal>()`.
2. **Lookahead**: peek 1–2 tokens; use predicate functions (e.g., `is_start_of_item()`).
3. **Operator Precedence**: Pratt parsing or precedence climbing for expressions.

### 2.3. AST Representation

* **Node Types**: `Node<T>` wrapping a `T` payload plus `Span`.
* **Enum Variants**: `AstItem`, `AstStmt`, `AstExpr`, each containing variant-specific structs.
* **Ownership**: use `Box` for recursion; `Vec` for sequences.

### 2.4. Error Recovery & Diagnostics

* **Single Token Insertion/Deletion**: upon mismatch, try skipping unexpected token or inserting a default.
* **Panic Mode**: synchronize at statement or block boundaries (`;`, `{`, `}`) after error.
* **Rich Diagnostics**: collect multiple `ParseError` instances, each with suggestions (e.g. "did you mean 'let'?" ).

### 2.5. Testing

* Grammar coverage tests mapped to each production.
* Golden-file tests: sample `.tl` files with expected AST dumps.

---

## 3. AST → TIR Lowering

* **Purpose**: transform high-level AST nodes into a simpler, SSA‑based Typed IR.

* **Passes**:

    1. **Name Resolution**: link identifiers to symbol table entries.
    2. **Type Checking**: annotate nodes with concrete types; error on mismatches.
    3. **Control-Flow Lowering**: convert loops/conditionals into basic blocks & jumps.
    4. **Desugarings**: sugar like `for`/`match` → primitives.

* **Error Propagation**: abort on binding/type errors, return `CompileError`.

---

## 4. Extensibility & Plugins

1. **Custom Syntax Plugins**: allow AST visitors to inject or transform nodes before lowering.
2. **Alternate Parsers**: support a `nom`‑based or `lalrpop` grammar under feature flags.
3. **Language Dialects**: enable syntax extensions via registry of keyword sequences.

---

## 5. Future Roadmap

* **Incremental Lex/Parse**: cache token streams and parse trees for editor integrations.
* **IDE Hooks**: emit position-based emissions for real‑time diagnostics and completions.
* **Spec-driven Grammar Tests**: auto‑generate tests from a formal grammar definition.

*End of Frontend Architecture (Lexing & Parsing).*

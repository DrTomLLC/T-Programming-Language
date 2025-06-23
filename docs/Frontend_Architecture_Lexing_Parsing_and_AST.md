# Frontend Architecture: Lexing, Parsing, and AST

The **Frontend** of the T compiler is responsible for transforming raw source code into a structured **Abstract Syntax Tree (AST)**. This AST serves as the basis for subsequent IR lowering, analysis, and code generation.

---

## 1. Lexical Analysis (Scanner)

* **Input**: UTF-8 source text, file paths, encoding metadata.
* **Responsibilities**:

    * *Tokenization*: Convert character stream into a sequence of tokens (identifiers, keywords, literals, operators, punctuation).
    * *Trivia Capture*: Preserve whitespace, comments, and documentation comments for tooling and formatting.
    * *Location Tracking*: Record line/column spans for each token for precise error reporting.
    * *Unicode Support*: Handle normalized source sequences, combining characters, wide characters.
* **Design Details**:

    * Table-driven state machine for speed.
    * Configurable token categories (future support for custom operators).
    * Streaming API to allow incremental lexing for large files and IDE integration.

## 2. Parsing (Parser)

* **Input**: Token stream with lookahead buffer.
* **Responsibilities**:

    * *Grammar*: LL(\*) or Pratt parser combining top‑down recursive descent with operator precedence.
    * *AST Construction*: Build nodes for expressions, statements, declarations, patterns.
    * *Error Recovery*: Synchronize on statement boundaries to continue parsing after errors.
    * *Macros & Metaprogramming*: Expand simple textual macros or annotation‑driven rewrites at parse time.
* **Design Details**:

    * Modular grammar files for easy extension by plugins.
    * Backtracking minimized; prefer predictive parsing with adaptive lookahead.
    * Configurable precedence table for user‑defined operators.
    * Hooks for plugin‑provided syntax extensions (e.g., domain‑specific constructs).

## 3. Abstract Syntax Tree (AST)

* **Core Nodes**:

    * **Module**: Contains imports, exports, top‑level declarations.
    * **Declarations**: `fn`, `struct`, `enum`, `const`, `type`, `trait`.
    * **Statements**: `let`, `if`, `while`, `for`, `match`, `return`, `expr`.
    * **Expressions**: Literals, variables, function calls, method calls, operator applications, closures, async/await.
    * **Patterns**: Destructuring for `match` and `let` bindings.
    * **Attributes**: Attach metadata to items (e.g., `#[inline]`, `#[test]`).
* **Metadata & Attachments**:

    * Source location spans (start/end byte offsets).
    * Comment/documentation annotations linked to nodes.
    * Type placeholders for early name resolution.
* **Ownership & Mutability**:

    * AST is an owned, tree‑shaped data structure with indexed arenas for nodes.
    * Borrow‑checking of AST references is enforced later; frontend only ensures no dangling pointers.

## 4. Visitors and Transformers

* **Visitor Pattern**:

    * Read‑only traversal for analysis passes (documentation extraction, syntax formatting).
* **Transformer Pattern**:

    * In‑place or functional rewriting of AST (macro expansion, desugaring, normalization).
* **Plugin Integration**:

    * Register custom visitors/transformers via plugin API.
    * Control ordering and interaction through pass manager.
* **Safety & Isolation**:

    * AST integrity ensured by snapshotting before major transformations.
    * Change detectors to avoid duplicate work.

## 5. Error Handling & Diagnostics

* **Lexing Errors**: Invalid characters, unterminated literals, indents.
* **Parsing Errors**: Unexpected tokens, missing punctuation, unmatched braces.
* **Recovery**: Insert synthetic tokens to continue parsing; aggregate multiple errors in one run.
* **Diagnostic System**:

    * Rich error messages with primary/secondary spans and notes.
    * Suggestions for fixes (typo hints, missing imports).
    * Integration with IDE for underlines and quick‑fix actions.

## 6. Extensibility & Customization

* **Grammar Plugins**: Dynamically register new keywords or syntax forms.
* **AST Annotation Plugins**: Attach custom metadata or perform domain‑specific analyses.
* **Configurable Flags**: Enable/disable language features or experimental syntax via `#cfg`.
* **Internationalization**: Support localized error messages.

## 7. Future Enhancements

* **Incremental Parsing**: Re‑use unchanged AST fragments for faster IDE feedback.
* **Syntax Trees**: Expose raw syntax tree to support code editors and formatters.
* **WebAssembly Frontend**: Support parsing in the browser for interactive tutorials.
* **Language Server Integration**: Provide on‑the‑fly parsing and diagnostics via LSP.

---

*(End of Frontend Architecture section)*

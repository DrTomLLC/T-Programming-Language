# Frontend Architecture — TIR Lowering

This section describes how the Typed Intermediate Representation (TIR) is constructed and manipulated, bridging the AST and backend codegen.

---

## 1. TIR Overview

* **Representation**: TIR is a SSA‑based, typed IR where each value is assigned exactly once.
* **Modules & Items**: A `TirModule` contains global functions, types, and constants.
* **Basic Blocks & CFG**: Functions are subdivided into `BasicBlock`s connected via explicit jumps.
* **Typed Values**: All TIR values carry a `Type` (primitive, composite, function, pointer).

---

## 2. TIR Data Structures

### 2.1. `TirModule`

* Fields: `Vec<TirFunction>`, `Vec<TirGlobal>`, `Vec<TirTypeDef>`.

### 2.2. `TirFunction`

* Signature: name, parameters `(name, Type)`, return `Type`.
* Body: entry `BasicBlock`, map of block IDs to `BasicBlock`.
* Metadata: calling convention, linkage.

### 2.3. `BasicBlock`

* Unique ID.
* Sequence of `Instruction`s.
* Terminator: `Terminator` enum (e.g., `Branch{cond, then, else}`, `Return(value)`).

### 2.4. `Instruction`

* Opcodes: binary ops, unary ops, `Call`, `Load`, `Store`, `Alloca`, `GetElementPtr`, etc.
* Each instruction produces zero or one SSA value (assign new `ValueId`).

### 2.5. `Type`

* Primitive: `Int(width)`, `Float(width)`, `Bool`.
* Composite: `Struct(name, fields)`, `Enum(name, variants)`, `Array(elem, len)`, `Slice(elem)`.
* Function pointers, pointers, mutable/immutable qualifiers.

---

## 3. Lowering Passes

### 3.1. Name Resolution & Symbol Binding

* Traverse AST, assign each declaration a unique TIR symbol.
* Populate symbol tables per scope.

### 3.2. Type Checking & Inference

* Check AST expressions, compute concrete types.
* Insert explicit casts and coercions in TIR.
* Reject ill‑typed constructs with `CompileError::TypeMismatch`.

### 3.3. Control‑Flow Construction

* Translate high‑level control constructs:

    * `if`/`else` → conditional branches.
    * `while`/`for` → loops with blocks for header, body, exit.
    * `match` → switch‑like multi‑way branch with pattern binding.

### 3.4. Expression Lowering

* Flatten nested expressions, assign temporaries.
* Desugar operators (`==`, `+`, `&&`, etc.) into primitive instructions.

### 3.5. Memory & Stack Allocation

* Lower local variable declarations to `Alloca` in entry block.
* Promote simple locals to registers when SSA allows.
* Insert `Load`/`Store` for pointers and references.

---

## 4. Optimizations on TIR

* **Constant Folding**: evaluate compile‑time constant expressions.
* **Dead Code Elimination**: remove unreachable blocks and unused values.
* **Simple Inlining**: inline small functions under feature flag.
* **Canonicalization**: normalize commutative operands.

---

## 5. Extensibility & Plugins

* Hook points before/after each lowering pass.
* Custom `TirVisitor` trait for transformations.
* Support alternative IR representations under feature flags.

---

## 6. Future Roadmap

* **SSA Construction**: integrate advanced algorithms (e.g., pruned SSA).
* **Memory SSA**: model memory operations as SSA values.
* **Type‑Driven Optimizations**: leverage high‑level type info.
* **Incremental Lowering**: reuse TIR between incremental compilations.

*End of Frontend Architecture (TIR Lowering).*

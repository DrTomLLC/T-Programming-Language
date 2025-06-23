# Intermediate Representation (IR) Architecture: TIR and Lowering

The **Intermediate Representation (IR)** is the bridge between the high‑level AST and the backend code generation. In T’s compiler pipeline, we define multiple IR levels—High‑level IR (HIR), Mid‑level IR (MIR), and Typed IR (TIR)—each catering to different analysis and optimization passes.

---

## 1. IR Hierarchy & Decomposition

* **HIR (High‑level IR)**

    * Close to AST shape but simplified (e.g., desugared `for`‑loops into `while`, explicit `match` cases).
    * Retains rich syntactic constructs for early analyses (pattern exhaustiveness, macro hygiene).

* **MIR (Mid‑level IR)**

    * A control‑flow graph of basic blocks and simple statements (assignments, jumps).
    * No nested expressions: broken into three‑address code.
    * Unresolved generic types; type placeholders remain.

* **TIR (Typed IR)**

    * MIR with fully resolved types and monomorphized generics.
    * Strongly typed, ready for borrow‑checking and ownership analyses.
    * Canonicalized call sites and VTable references for trait objects.

---

## 2. IR Data Structures

* **Module Representation**

    * Contains a collection of functions, types, global constants, and metadata.
    * Indexes into global symbol table for resolution and linkage.

* **Function & BasicBlock**

    * **Function**: entry/exit block, signature (arguments, return types), attributes (inline hints).
    * **BasicBlock**: list of `Instruction`s ending with a terminator (`Branch`, `Return`, `Call`).

* **Instruction Set**

    * **Arithmetic & Logic**: `Add`, `Sub`, `Mul`, `Div`, `And`, `Or`, etc.
    * **Memory Ops**: `Alloca` (stack), `Load`, `Store`, `GetElementPtr` (struct/array indexing).
    * **Control Flow**: `Jump`, `CondJump`, `Switch`.
    * **Call & Invoke**: normal function calls and error‑handling variants.
    * **Intrinsic & Built‑ins**: vector ops, atomic primitives, builtin math functions.

* **Types**

    * Primitive: integers (i8..i128), floats (f32, f64), `bool`, `char`.
    * Composite: arrays, structs, tuples, enums (tagged unions).
    * Pointers & References: raw pointers, borrow‑checked references.
    * Generics / Monomorphized types.

---

## 3. Lowering Pipeline

1. **AST → HIR**

    * Desugar syntactic sugar, inline macros, lower complex patterns.
2. **HIR → MIR**

    * Break nested expressions, lift variable declarations, normalize CFG structure.
    * Introduce temporaries for intermediate values.
3. **MIR → TIR**

    * Perform type inference/monomorphization.
    * Resolve trait dispatch, vtable layout, and drop glue insertion.
    * Insert borrow‑check annotations (liveness, mutability checks).

**Design Notes**:

* Each pass produces an explicit data snapshot; previous IR can be cached for incremental updates.
* Errors that surface in lowering (e.g., type mismatches, missing trait impls) are reported with IR spans.

---

## 4. Control‑Flow Graph & SSA Form

* **CFG Representation**:

    * Graph of `BasicBlock` nodes and directed edges.
    * Edge metadata for exception/unwind paths.

* **SSA (Static Single Assignment)**:

    * Introduce φ‑nodes at merge points.
    * Simplifies data‑flow analyses (constant propagation, dead code elimination).
    * Optionally demoted after early optimizations to simplify codegen.

---

## 5. Pass Infrastructure

* **Pass Manager**:

    * Schedule and coordinate analysis passes (liveness, alias analysis) and transformation passes (inlining, loop‑unroll).
    * Dependencies between passes ensure required analyses are computed first.

* **Analysis vs. Transformation**:

    * Analyses (read‑only): collect metadata, compute dominance, compute borrow graphs.
    * Transformations (mutable): inlining, constant folding, dead code elimination, loop optimizations.

* **Plugin Hooks**:

    * Plugins can register custom passes via the backend API.
    * Control ordering by specifying before/after markers for core passes.

---

## 6. Diagnostics & Debug Info

* **IR‑Level Errors**:

    * Report monomorphization failures, intrinsics misuse, borrow errors.
    * Include original AST spans and IR spans for precise highlighting.

* **Debug Metadata**:

    * Embed source file, line, column, variable mapping into IR for DWARF generation.
    * Support for breakpoints and stepping in IDEs.

---

## 7. Extensibility & Future Work

* **IR Extensions**:

    * Support for domain‑specific instructions (e.g., GPU kernels, DSP ops).
    * Pluggable instruction sets for alternative backends (WebAssembly, mobile ABI flavors).

* **Optimizations**:

    * Profile‑guided, link‑time, and interprocedural optimizations.
    * Vectorization and parallelization annotations in IR.

* **Incremental IR Reuse**:

    * Cache and reuse IR fragments across edits for fast IDE feedback.
    * Fine‑grained invalidation using IR dependency graphs.

---

*(End of IR Architecture section)*

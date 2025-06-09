# Section 16: T‑Intermediate Representation & Semantic Model

## 16.1 Overview
TIR sits at the heart of the compiler, bridging high‑level AST and low‑level codegen. It must be:
- **Rich enough** to express all language constructs (control flow, functions, types, metadata)
- **Stable** so plugins and analyses can depend on a consistent layout
- **Optimizable** via passes (inlining, constant folding, dead code elimination)

## 16.2 TIR Modules & Items
- **`Module`**: top‑level container for items, import/export lists, and module‑scoped metadata
- **`Item`** variants:
    - `Function { signature, body, visibility }`
    - `Global { name, type, initializer }`
    - `TypeDef { name, definition }`
    - `Import` / `Export` directives
    - `Const`, `Enum`, `Struct`, `Union`, `Trait`, etc.

## 16.3 Types & Signatures
- **Primitive types**: integers, floats, booleans, characters, strings
- **Composite types**: tuples, arrays, slices, pointers, references, generics
- **Function signatures**: parameter list, return type(s), calling convention, safety annotations
- **Metadata**: nullability, mutability, alignment, ABI tags

## 16.4 Expressions & Statements
- **Expression AST** in TIR:
    - `Call`, `MethodCall`, `Constructor`, `BinaryOp`, `UnaryOp`, `Cast`
    - `FieldAccess`, `Index`, `Slice`, `Path`
    - `Literal`, `Variable`, `Closure`, `BlockExpr`
- **Statement variants**:
    - `Let { pattern, initializer, mutable }`
    - `Assign`, `If`, `Match`, `Loop`, `While`, `For`, `Return`, `Break`, `Continue`
    - `UnsafeBlock`, `ExternBlock`, `InlineAsm`

## 16.5 Control‑Flow Graph (CFG)
- **BasicBlocks**: anchored by labels, end in terminators (`Jump`, `CondJump`, `Return`, `Unreachable`)
- **Edges** represent possible flow—used for liveness, dominators, and optimization analyses
- **Transformation passes** can lower structured constructs (`If`, `Match`) into CFG form

## 16.6 Name Resolution & Scoping
- **Symbol table** attached to each module & block
- **Lexical scoping** rules enforce shadowing, hygiene, and import resolution
- **Plugin hooks** can inject or override resolution behavior (e.g., macro expansion)

## 16.7 Type Checking & Inference
- **Constraint generation** during TIR lowering:
    - Generate equality/variance constraints for generics
    - Track lifetimes and borrow scopes
- **Unification & solver**: resolves type variables, reports errors with spans
- **Monomorphization** plan for generics—evaluate trade‑offs between code size and compile time

## 16.8 Semantic Analyses & Validation
- **Enum exhaustiveness**, **match safety**
- **Dead code detection**, **unused imports/variables**
- **Borrow‑checker**: enforces aliasing/mutation rules via region inference
- **Plugin APIs** allow custom linting, metrics, or code transforms at this IR stage

## 16.9 Persisting & Visualizing TIR
- **Serialization**: JSON or binary for caching incremental builds
- **Debug dumps**: human‑readable pretty‑print for diagnostics
- **Graphviz exports** for CFG and call graphs  

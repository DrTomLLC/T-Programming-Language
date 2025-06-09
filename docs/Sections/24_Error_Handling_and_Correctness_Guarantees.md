# Section 24: Error Handling & Correctness Guarantees

## 24.1 Philosophy & Goals
- **Eliminate whole classes of bugs** at compile time: no null/dangling pointers, no data races, no buffer overruns.
- **Guarantee semantic correctness** through formal methods, contracts, and rich type‐level encoding of invariants.
- **Provide “never‐panic” defaults**, with all recoverable errors surfaced as typed `Result<…>` values.

## 24.2 Type‐System & Ownership
- **Affine/Linear Types** for resource safety: ensure handles, locks, and buffers can’t be mis‐used.
- **Non‐nullable references** by default; `Option<T>` must be explicit.
- **Immutable by default**, with explicit `mut` or capability types for mutation.

## 24.3 Contracts & Assertions
- **Pre/post conditions** on functions (`requires`/`ensures`) checked statically when possible, and optionally at runtime.
- **Refinement types** for numeric and textual constraints (e.g. `Port(0..65535)`, `Email<Regex>`).
- **Checked array bounds**; out‐of‐bounds is a compile‐error, never UB.

## 24.4 Algebraic Effects & Error Algebra
- **Result‐style error propagation** with zero‐cost async/await integration.
- **Effect systems** track which functions may fail, which may perform I/O, allocate, or block.
- **Typed error hierarchies** so you can `match` exhaustively and never see an unknown error.

## 24.5 Formal Verification & SMT Integration
- **SMT‐backed proof obligations** generated for critical modules (e.g. cryptography, safety‐critical code).
- **“Proof‐by‐example” harnesses** automatically generate invariants and check them against a solver.
- **Lightweight proof annotations** embed inline lemmas that the compiler discharges.

## 24.6 Static & Dynamic Analysis
- **Built‐in data‐flow, taint, and alias analysis** as part of the front end.
- **Fuzz‑testing harness generation** from function signatures & types.
- **Sanitizers (ASan, UBSan) integration** controlled via feature flags for debug builds.

## 24.7 Rich Diagnostics & Fix‑Its
- **Multi‐span error messages** that show related code locations together.
- **Automated code actions**: “Add `.unwrap_or(default)` here”, “Change `&T` to `Option<&T>`”, etc.
- **Interactive error digests** in IDEs: click to see counter‐examples or runtime contract failures.

## 24.8 Reproducible & Deterministic Builds
- **Content‑hashized artifacts** for caches, preventing “it builds on my machine” issues.
- **Checksum‑checked dependencies** (lockfiles + signing) for supply‐chain integrity.
- **Deterministic codegen** across LLVM/Cranelift backends.

## 24.9 Safe FFI & Plugin Boundaries
- **Boundary‐checked ABI layers** auto‐generate safe wrappers for C, Zig, or other backends.
- **Memory ownership contracts** at FFI: declare “borrowed” vs “owned” pointers.
- **Capability tokens** passed to plugins to restrict what they may do at runtime.

## 24.10 Testing & CI Guarantees
- **Property‑based tests** scaffolded automatically from function signatures.
- **Coverage‐driven test recommendations**: compiler suggests untested code paths.
- **Integrated mutation testing** to ensure test suite robustness.

---

> By combining a modern, expressive type system with formal verification hooks, powerful diagnostics, and strict build determinism, T‑Lang aims to make entire categories of bugs—memory safety, concurrency races, undefined behavior—impossible, and deliver provably correct programs end‑to‑end.  

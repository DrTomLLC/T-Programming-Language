# Section 22: Energy & Performance Modes

## 22.1 Overview
T‑Lang supports selectable “modes” that tune compiler and runtime behavior for different trade‑offs between energy consumption, performance, code size, and responsiveness.

- **Performance Mode**
    - Maximum optimization level (`-O3` or `--release`)
    - Aggressive inlining, vectorization, unrolling
    - High power draw, best raw throughput
- **Balanced Mode**
    - Moderate optimization (`-O2`)
    - Reasonable inlining and unrolling
    - Default for general workloads
- **Energy‑Saving Mode**
    - Low optimization level (`-O1` or `--energy`)
    - Avoids heavy vectorization/unrolling
    - Reduces CPU/GPU frequency scaling, lowers power draw
- **Size‑Optimized Mode**
    - Focus on `-Os`
    - Strips debug info, dead‑code elimination
    - Minimizes binary footprint (critical for embedded)

---

## 22.2 Selection Mechanisms

### 22.2.1 Compiler Flags
- `--mode=performance` → `-C opt-level=3 -C target-cpu=native`
- `--mode=balanced`    → `-C opt-level=2`
- `--mode=energy`      → `-C opt-level=1 -C energy-aware=true`
- `--mode=size`        → `-C opt-level=s -C strip=symbols`

### 22.2.2 Configuration File
```toml
[profile.performance]
opt-level = "3"
energy-aware = false

[profile.energy]
opt-level = "1"
energy-aware = true

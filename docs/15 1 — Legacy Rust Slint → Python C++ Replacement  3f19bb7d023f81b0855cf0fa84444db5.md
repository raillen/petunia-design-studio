# 15.1 — Legacy Rust/Slint → Python/C++ Replacement Matrix

# Replacement rules

| Legacy | New authority | Status |
| --- | --- | --- |
| Rust application/core | C++23 domain/engines + Python application layer | Reimplemented |
| Slint shell | PySide6/Qt Widgets shell | Reimplemented |
| Rust bridge types | nanobind typed facade | Reimplemented |
| Rust crates | CMake C++ libraries + Python packages | Reimplemented |
| cargo xtask | tooling CLI / CMake presets / Python task runner | Reimplemented |
| Vello/wgpu decision | renderer backend ADR/spike | Re-evaluated |
| mlua embedded plugins | trusted Python in-process + third-party Python process + optional WASM | Reimplemented/Expanded |
| Rust semantic core rules | C++ canonical core | Preserved |
| .PTND | .PTND open package | Preserved/Expanded |
| Actions/Commands | same semantic architecture | Preserved |
| Design + Photo | same product model | Preserved |
| Surface | same abstraction | Preserved |
| semantic color | C++ ColorEngine + adapters | Preserved |
| MCP semantic automation | Python transport + C++ semantic services | Preserved/Expanded |

# Migration rule

No direct line-by-line port requirement. Preserve behavior/contracts and rebuild internals idiomatically for target language.

# Compatibility

Native format migrations must distinguish old **schema** compatibility from old implementation language. PTND documents should not care whether writer was Rust or C++ if schema semantics compatible.

# Historical code

Old code can serve as fixture/oracle/reference but must not impose Rust-shaped abstractions on C++/Python.
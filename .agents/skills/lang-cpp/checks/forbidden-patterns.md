# C++ Forbidden Patterns Checklist

| Pattern | Alternative | Rationale |
|---|---|---|
| `reinterpret_cast<T*>` | `std::bit_cast<T>` or escape-hatch | Violates type safety and strict aliasing |
| `(T)value` (C-style cast) | `static_cast<T>`, `std::bit_cast<T>` | Indiscriminate conversion masks bugs |
| `new T`, `delete ptr` | `std::make_unique<T>()` | Naked heap allocations cause memory leaks |
| `malloc`, `free`, `realloc` | `std::vector<T>`, `std::unique_ptr` | Bypasses constructor/destructor invariants |
| `ptr++`, `*(ptr + offset)` | `std::span<T>`, iterators | Raw pointer arithmetic causes buffer overflows |
| `goto <label>` | Structured loops, early returns, RAII | Spaghetti control flow obscures lifetimes |
| `// nolint` without registration | Formally registered waiver | Hides technical debt and safety hazards |
| `#define CONSTANT 42` | `inline constexpr auto Constant = 42;` | Macro lacks scope, type safety, and debug symbols |

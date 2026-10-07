# Modern C++ Memory Model, Lifetimes & Ownership

## 1. Ownership Taxonomy
- **Exclusive Ownership**: Represented strictly by `std::unique_ptr<T>`. Transfer ownership via `std::move()`.
- **Shared Ownership**: `std::shared_ptr<T>` with `std::weak_ptr<T>` to break cyclic graphs. Used only when multiple owners independently control lifecycle.
- **Non-Owning Views**: `std::span<T>` for contiguous sequences, `std::string_view` for string data. Non-owning views must never outlive their referenced storage.
- **Value Semantics**: Prefer pass-by-value and move semantics for lightweight types, `const T&` for read-only large types.

## 2. Lifetime Invariants
- Never return a reference or non-owning view to a local automatic variable.
- Never bind a string_view to a temporary `std::string` return value.
- Clear containers when storing pointers or references before destroying parent owners.

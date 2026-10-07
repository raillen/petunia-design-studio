# Non-Negotiable AI Coding Contract for C++

1. **Deterministic Design**: Write explicit types with concepts. Never generate speculative abstractions or templated complexity without concrete functional need.
2. **Zero Naked Pointers**: Never return `T*` or accept `T*` unless interfacing with external unmodifiable C APIs within a dedicated quarantine wrapper.
3. **No Hidden State**: Avoid global or static mutable variables. Explicitly pass state or context.
4. **Const Correctness**: Mark all non-mutating member functions `const`. Use `constexpr` and `consteval` wherever computations can occur at compile-time.
5. **No Blind Suppressions**: The AI agent MUST NOT add `// NOLINT` or `#pragma GCC diagnostic ignored` to silence compiler or linter errors. Every error must be resolved at the root cause.

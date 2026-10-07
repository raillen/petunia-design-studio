---
name: lang-cpp
description: Modern C++ (C++20/C++23) engineering, RAII ownership, banned raw allocations, sanitizers (ASan/UBSan), static analysis, and zero-defect AI implementation contract.
---

# Modern C++ Engineering & Extreme Safety Contract

## 1. Core Paradigm & Standards
- Target **C++20** or **C++23**. Legacy pre-C++20 idioms are prohibited unless backward compatibility is explicitly mandated.
- Follow **RAII (Resource Acquisition Is Initialization)** unconditionally. All resource lifecycles (memory, locks, file descriptors, sockets) must be tied to object lifetimes.
- Zero raw pointer manipulation. Naked `new`, `delete`, `malloc`, and `free` are strictly forbidden. Use `std::unique_ptr` for exclusive ownership, `std::shared_ptr` only when ownership is genuinely shared, and `std::span` / `std::string_view` for non-owning views.

## 2. Forbidden Constructs & Immediate Failures
The following patterns fail compilation and quality gates immediately:
- `reinterpret_cast` (unless formally registered in `.prumo/escape-hatches.json` with safety invariants).
- C-style casts `(type)val`. Use `static_cast`, `std::bit_cast`, or type traits.
- Raw pointer arithmetic (`ptr++`, `*(ptr + i)`). Use iterators or `std::span`.
- Uninitialized variables. Always initialize: `int count = 0;` or auto `{}`.
- C-style macro constants (`#define BUFFER_SIZE 1024`). Use `inline constexpr auto BufferSize = 1024;`.
- `goto` statements. Use structured control flow.
- Uncaught exceptions. Use `std::expected<T, E>` (C++23) or `std::optional<T>` for fallible operations.

## 3. Universal AI Implementation Loop for C++
1. **Red Phase**: Write unit tests (Catch2, GoogleTest, or Boost.UT) specifying exact invariants, boundaries, and failure cases before implementation.
2. **Green Phase**: Implement minimal idiomatic C++ satisfying tests using value semantics, smart pointers, and standard containers.
3. **Hardening Phase**:
   - Compile with `-Wall -Wextra -Wpedantic -Wconversion -Wshadow -Werror`.
   - Run static analysis: `clang-tidy --warnings-as-errors=*` and `cppcheck`.
   - Run sanitizers: AddressSanitizer (`-fsanitize=address`), UndefinedBehaviorSanitizer (`-fsanitize=undefined`), and ThreadSanitizer (`-fsanitize=thread`).
   - Run memory leak detection (LeakSanitizer / Valgrind).
4. **Evidence Generation**: Output normalized test and sanitizer results conforming to `schemas/evidence.schema.json`.

## 4. Escape-Hatch Quarantine Policy
When low-level hardware or foreign function interfaces (FFI) require escaping safety rules:
- Isolate the escape hatch within a dedicated translation unit or inline boundary.
- Formally register the hatch in `.prumo/escape-hatches.json` or annotate with `// PRUMO:ESCAPE_HATCH[id]`.
- Provide exhaustive sanitizer and differential unit tests covering the boundary.

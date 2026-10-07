# 12.2 — C++23 Safety, Ownership, Error Handling, Memory & Concurrency Discipline

# Standard

C++23 baseline with clang/gcc/MSVC support matrix. Warnings-as-errors for project code.

# Ownership

Prefer values and Rule of Zero. unique_ptr = unique heap ownership. shared_ptr only when ownership is genuinely shared and lifecycle cannot be represented more simply. weak_ptr breaks observational cycles. Raw pointer/reference/span = non-owning and must not outlive owner.

# Allocation

No naked new/delete in feature code. Hot paths profile allocation before custom arena/pool. Per-frame transient arenas are permitted only with measured benefit and clear reset lifetime.

# Containers

Choose by access pattern. Stable IDs never equal vector indexes. Reserve/batch where evidence shows churn. Avoid pointer invalidation assumptions across vector growth.

# Error model

Exceptions may be used internally where project policy permits but cannot cross C ABI/binding boundary uncontrolled. expected/result style preferred for anticipated failures in parsing/IO/validation.

# Threading

std::jthread + stop_token for owned threads. Locks scoped RAII. Lock ordering documented for multi-lock subsystem. Avoid callbacks while locks held. Atomics use documented memory order.

# Immutability

Worker jobs consume immutable snapshots/value objects. Shared mutable canonical document is not concurrently modified from arbitrary jobs.

# Safety tooling

ASan + UBSan mandatory on supported CI; TSan dedicated jobs; static analysis; fuzz parsers/geometry. Release hardening flags and platform mitigations audited.

# Performance

SIMD/cache optimization only after profiler evidence. Intrinsics behind scalar reference implementation/test where practical.

# Bindings

No borrowed STL reference returned to Python if owner can mutate/free. Large buffers expose lifetime capsule/owner. GIL release wrappers cannot touch Python API until reacquired.

# Shutdown

Explicit ordering: stop producers -> cancel jobs -> join -> destroy render/device -> sessions -> bindings. Avoid static destructor order dependencies.
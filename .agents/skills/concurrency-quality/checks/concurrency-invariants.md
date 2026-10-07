# Concurrency Invariants Checklist

- [ ] Shared mutable state is guarded by mutexes or lock-free constructs.
- [ ] Goroutines/Threads have guaranteed termination paths.
- [ ] Timeouts applied to all network/blocking operations.
- [ ] Lock scopes are minimized to prevent contention.

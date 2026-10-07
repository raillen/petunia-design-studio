# Concurrency & Async Checklist
- [ ] No unconstrained unbounded channels or memory queues
- [ ] Lock hierarchy is strictly ordered to prevent deadlocks
- [ ] Lock acquisition scopes are minimal (no I/O under lock)
- [ ] Cancellation and timeouts are supported on all blocking operations
- [ ] Tasks handle shutdown signals and drain cleanly

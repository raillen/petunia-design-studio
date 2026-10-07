---
name: concurrency-quality
description: Races, deadlocks, cancellation, backpressure, leaked tasks, unbounded channels, lock scope, resource cleanup, timeouts, ownership, shutdown
---
# Concurrency Quality

## 1. Data Race Prevention
Eliminate data races by strictly controlling shared mutable state. Prefer message passing (channels/actors) or immutable data structures. When using shared memory, enforce exclusive access using mutexes. Use race detectors during CI runs.

## 2. Deadlock Avoidance
Prevent deadlocks by establishing a strict, global lock acquisition order. Never acquire multiple locks simultaneously if possible. Use timeout-based lock acquisition. Avoid calling foreign, untrusted code while holding a lock.

## 3. Task Cancellation
Implement robust cooperative cancellation mechanisms (e.g., Context in Go, CancellationToken in .NET). Long-running tasks must periodically check for cancellation requests and terminate cleanly, releasing resources promptly.

## 4. Backpressure Mechanisms
Implement backpressure to prevent fast producers from overwhelming slow consumers. Use bounded queues, rate limiting, or load shedding. When a system is overloaded, it must signal upstream systems to slow down or drop requests cleanly.

## 5. Task Leakage Prevention
Ensure every spawned task or goroutine has a guaranteed termination path. Avoid unbounded blocking operations. Use wait groups or structured concurrency patterns to track and await the completion of all child tasks.

## 6. Channel and Queue Management
Avoid unbounded channels or queues, which can lead to Out-Of-Memory (OOM) crashes under load. Size queues appropriately based on expected throughput and latency requirements. Handle queue full conditions gracefully (block, drop, or return error).

## 7. Lock Scoping
Keep critical sections (code executed while holding a lock) as absolutely short as possible. Do not perform slow I/O or blocking operations while holding a lock. Release the lock immediately after updating the shared state.

## 8. Resource Cleanup
Guarantee resource cleanup using `defer`, `finally`, or RAII patterns. Ensure file handles, database connections, and network sockets are closed even if a concurrent task panics or is cancelled unexpectedly.

## 9. Strict Timeouts
Apply strict timeouts to every blocking operation, especially network calls. A system without timeouts will eventually hang indefinitely when an external dependency fails. Cascading failures are often caused by missing timeouts.

## 10. Graceful Shutdown
Implement graceful shutdown sequences. When the application receives a termination signal, stop accepting new requests, allow in-flight requests to complete within a timeout, and then safely shut down background tasks and release resources.


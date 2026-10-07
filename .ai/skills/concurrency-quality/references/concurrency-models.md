# Concurrency Models in Python

## 1. Asyncio (Coroutines)
- **Best For**: I/O-bound tasks (network requests, database queries, file system operations).
- **How it works**: Uses an event loop and cooperative multitasking. A single thread runs the loop, pausing execution at `await` boundaries to let other tasks run.
- **Pros**: Very lightweight, can handle thousands of concurrent connections.
- **Cons**: Blocking the event loop (e.g., with a CPU-heavy calculation or a synchronous network call) freezes the entire application.

## 2. Threading
- **Best For**: I/O-bound tasks when using synchronous libraries that don't support Asyncio.
- **How it works**: OS-level threads. In standard CPython, the Global Interpreter Lock (GIL) prevents true parallel execution of Python bytecodes.
- **Pros**: Easy to use with existing synchronous code.
- **Cons**: High memory overhead compared to coroutines, context switching cost, race conditions require careful locking.

## 3. Multiprocessing
- **Best For**: CPU-bound tasks (data crunching, image processing).
- **How it works**: Spawns entirely new Python processes, each with its own GIL and memory space.
- **Pros**: True parallelism on multi-core machines. Bypasses the GIL constraint.
- **Cons**: High startup cost, heavy memory usage, complex IPC (Inter-Process Communication) to share state.

## Concurrency Guidelines
- Avoid shared state whenever possible. If state must be shared, use appropriate synchronization primitives (Locks, Semaphores, Queues).
- Be extremely careful about deadlock when acquiring multiple locks. Always acquire locks in a consistent order.

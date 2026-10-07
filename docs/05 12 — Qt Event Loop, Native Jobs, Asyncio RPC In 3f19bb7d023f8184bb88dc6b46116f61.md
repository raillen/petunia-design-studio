# 05.12 — Qt Event Loop, Native Jobs, Asyncio/RPC Integration & Responsiveness Contract

# Main thread rule

All QWidget/QWindow mutation happens on Qt GUI thread.

# Native jobs

C++ JobScheduler executes heavy work and reports thread-safe progress/completion messages. Qt dispatcher posts compact updates to main event loop.

# Python async

MCP/plugin RPC may use asyncio or async libraries in dedicated integration strategy. Do not block or nest arbitrary event loops inside QAction callbacks.

# Bridging

Use explicit AsyncService/Qt dispatcher boundary or separate I/O thread/process. Results return as typed messages/futures and revalidate session/revision before applying.

# Responsiveness

No synchronous operation on GUI thread that can exceed small interactive budget without strong evidence. File parse/export/filter/thumbnail jobs background by default.

# Modal loops

Avoid nested modal event loops for long-running work. Dialogs can be window-modal but asynchronous job continues through normal event loop.

# Progress

Throttle/coalesce to human-visible update rate; never emit 1000 signals/sec to progress bar.

# Shutdown

Async services stop accepting work, cancel, drain bounded messages and join before QApplication teardown/native core destruction.
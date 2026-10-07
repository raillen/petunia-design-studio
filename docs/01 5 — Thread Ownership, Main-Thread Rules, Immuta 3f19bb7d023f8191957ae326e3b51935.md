# 01.5 — Thread Ownership, Main-Thread Rules, Immutable Jobs & Locking Strategy

# Thread domains

Qt GUI thread owns widgets/window input and Python UI controllers.

Document mutation executor serializes canonical commits; initially may be GUI/application thread with short commits or dedicated application serialization mechanism by ADR.

JobScheduler workers own heavy computation on immutable snapshots.

Renderer thread/queue behavior backend-specific behind contract.

# No shared arbitrary mutation

Workers never mutate DocumentStore directly. They return StagedResult/derived cache item keyed revision.

# Locks

Prefer immutability/message passing. Locks protect caches/queues/resource managers, not entire app. Lock order documented per subsystem; no Python callback or file/network blocking while lock held.

# Python

Heavy C++ calls release GIL. Worker threads cannot call Python C API unless explicitly reacquired on designated boundary, normally avoided. Qt widgets never touched outside GUI thread.

# Render synchronization

RenderSession consumes immutable scene/resources; document edits produce next scene generation rather than sharing mutable node tree under coarse lock.

# Shutdown

Stop external producers -> stop MCP/plugin writes -> cancel jobs -> close/save sessions -> drain events -> destroy views/renderer -> core -> Python/Qt teardown.

# Tests

TSan, forced shutdown during jobs, rapid document close/open, plugin event after revoke, renderer stale scene and deadlock watchdog.
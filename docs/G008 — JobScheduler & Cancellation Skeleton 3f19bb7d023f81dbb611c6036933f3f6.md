# G008 — JobScheduler & Cancellation Skeleton

# Goal

Create native bounded job system before expensive engines depend on ad-hoc threads.

# Depends

G003, G006.

# Primary

systems-architect.

# Skills

lang-cpp, concurrency-quality, memory-management, testing-quality.

# Deliverables

JobId; priority enum; JobScheduler worker pool; std::jthread/stop_token; JobHandle state; progress snapshot; cancellation; completion queue; stale generation metadata; shutdown sequence; Python facade.

# Invariants

No GUI access from worker; callbacks not invoked under locks; bounded queues; one cancellation state; scheduler destruction joins workers.

# Acceptance

Run/cancel jobs, priority behavior, completion delivery, clean shutdown and exception capture. Python can poll/receive compact completion without worker touching Python API.

# Tests

TSan, saturation, priority, cancel-before/start/mid, exception, shutdown with active jobs, repeated create/destroy.

# Non-goals

Specialized render scheduler or distributed jobs.
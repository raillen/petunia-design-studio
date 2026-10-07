# G08 — JobScheduler & Cancellation Skeleton

# Goal

Implement native bounded job execution before expensive features depend on ad-hoc threads.

# Depends

G03, G06.

# Authority

09.13, 05.12.

# Owner

systems-architect.

# Deliverables

JobId, priority lanes, std::jthread workers, stop_token cancellation, progress/result/error, bounded queue, completion dispatcher.

# Acceptance

Priority, cancellation, queue saturation, shutdown and stale-result tests; TSan clean; Python can observe/cancel JobHandle without worker invoking arbitrary Python.

# Non-goals

No renderer-specific jobs.
# 09.13 — Job System, Concurrency, Cancellation & Priority Scheduling

# Principle

Aubrieta must keep interaction responsive while geometry, raster processing, thumbnails, import/export, font discovery, indexing and color analysis run concurrently. Concurrency must never create multiple document mutation authorities.

# Execution domains

- **UI/application executor:** owns window/application state and coordinates Commands;
- **Document mutation lane:** serialized authoritative commits; may be the application executor or a dedicated single-owner service, but never concurrently mutated;
- **CPU job pool:** Rayon/data-parallel and bounded task work;
- **GPU queue:** rendering/compute submitted through renderer services;
- **I/O tasks:** bounded async/blocking adapters for filesystem/network-capable plugins.

# Job model

`JobId`, `JobKind`, priority, owner module, document revision/snapshot, cancellation token, progress channel, result channel, resource/memory estimate where relevant.

Priorities: Interactive Preview > Visible Evaluation > User-blocking Save/Open > Export/Import foreground > Background Index/Thumbnail > Maintenance.

# Cancellation

Cancellation is cooperative and checked at safe boundaries. A canceled job may discard derived output freely; a job that would partially mutate canonical state must stage changes and commit atomically only if not canceled.

# Stale-result policy

Jobs carry source revisions/fingerprints. On completion, stale derived results are discarded or revalidated. Background jobs never overwrite newer document state merely because they finish later.

# Backpressure

Queues are bounded or coalesced. Repeated thumbnail/invalidation requests for the same key collapse. Brush/input work can supersede old previews. Memory-heavy jobs declare budgets before allocation where possible.

# Progress

Jobs emit structured phases and optional fraction, never localized text directly. UI maps phases to `TextId`. Unknown duration uses indeterminate progress.

# Panic/failure containment

Background panics/errors become failed JobResult diagnostics; release builds should avoid taking down the whole app when failure can be isolated. GPU device loss is handled by renderer recovery path.

# Tests

Cancellation races, stale completion, priority inversion, queue saturation, disk-full/save cancellation, document close while jobs run, module disable with owned jobs, deterministic final state after randomized job completion order.

# Job state machine

Every externally observable job follows a small explicit lifecycle:

```
Created → Queued → Running → Succeeded
                    ↘ Failed
                    ↘ Cancelling → Cancelled
Queued ─────────────→ Cancelled
```

`Succeeded` means the result has been published/committed according to that JobKind, not merely that worker computation returned bytes. A stale derived computation can finish worker execution but resolve as `DiscardedStale`/non-success publication state rather than masquerading as current output.

# Job ownership

Every job has an `OwnerScope`:

- application;
- document session;
- window/workspace only when truly UI-session specific;
- module/plugin instance;
- export/import batch/subsystem.

Owner shutdown/disable automatically cancels or detaches jobs according to declared lifecycle. A job without an owner is invalid except a tiny set of explicitly application-global maintenance jobs.

# Job descriptor

Canonical metadata should include:

```
JobDescriptor
- JobId
- JobKind / semantic operation ID
- OwnerScope
- priority class
- document/snapshot revision or input fingerprints
- cancellation token
- deadline/timeout where policy applies
- estimated memory/IO/GPU weight when known
- progress schema
- trace/correlation ID
- stale-result publication policy
```

User-facing title/phase text is resolved through TextId outside worker code.

# Priority classes and fairness

Priority is a **class plus scheduler policy**, not a single arbitrary integer exposed to feature code.

Recommended order:

1. `InputCritical` — brush/input/interaction work required for immediate feedback;
2. `VisibleInteractive` — active viewport evaluation/layout;
3. `ForegroundBlocking` — Open/Save or user-waiting command phase;
4. `ForegroundLongTask` — import/export/batch explicitly launched;
5. `VisibleBackground` — histogram/current thumbnails/current asset view;
6. `Background` — indexing/previews/offscreen thumbnails;
7. `Maintenance` — compaction/scavenging/low-priority cleanup.

The scheduler prevents starvation with bounded aging/fair-share inside non-input classes. A continuous stream of thumbnails cannot starve Save; a continuous export cannot starve brush input. Feature modules cannot self-promote to `InputCritical` without a reviewed contract.

# Work domains

Do not collapse all work into one executor.

- Rayon/data-parallel CPU pool: bounded CPU-heavy pure computation;
- async runtime (Tokio selectively): coordination, timers, async I/O and brokered network where needed;
- blocking-I/O pool/`spawn_blocking`: filesystem/codecs/libraries that block threads;
- renderer/GPU submission service: device-owned operations;
- authoritative document mutation lane: commits only.

Never call blocking parser/file/FFI work on the GPUI event loop. Never hold a document/state lock across `.await`.

# CPU oversubscription policy

Libraries/internal algorithms must not each spawn uncontrolled thread pools. Aubrieta owns parallelism policy where practical. Rayon work submitted from jobs should respect global CPU budget; nested parallel work is profiled and limited. Plugin runtimes cannot create unbounded host threads.

# Concurrency-safe inputs

Background jobs consume:

- immutable snapshots/DTOs;
- Arc-owned immutable resources;
- content-addressed bytes;
- explicitly leased derived resources.

They do **not** borrow mutable DocumentStore/UI entities across threads.

# Result publication

A worker result is a proposal until publication validates:

1. owner still active;
2. cancellation not terminal;
3. input revision/fingerprint still acceptable;
4. target module/document/session still exists;
5. result resource limits/schema valid;
6. publication happens on the owning service lane.

Derived result publication updates caches. Canonical mutation result publication submits a Command to the mutation lane; worker code never directly commits.

# Stale policies

Each JobKind chooses one:

- `DiscardIfStale` — thumbnails/histograms/previews;
- `RevalidateThenPublish` — some indexes or target-ID operations;
- `SnapshotBoundResult` — export result remains valid for captured revision even if document advances;
- `RescheduleLatest` — visible evaluation where latest result is required.

The policy is part of JobKind metadata, not guessed at completion.

# Cancellation hierarchy

Cancellation tokens can form parent→child trees. Cancelling an export batch cancels outstanding child exports; cancelling one child need not cancel siblings unless batch policy says fail-fast. Document close cancels document-owned derived/background jobs while Save/close coordination follows 09.24.

Cancellation is idempotent. Multiple callers may request cancel safely.

# Cancellation latency

Each JobKind defines a maximum practical checkpoint interval/latency budget. Tight interactive work checks frequently; coarse codecs/FFI may only cancel between library calls and must document that limitation. UI may show `Cancelling…` when termination is not immediate.

# Critical sections

A job may enter a short `NonInterruptibleCommit` phase only for operations that would become corrupt if interrupted (e.g. atomic file replace). Cancellation requested during that phase is remembered and reported after the critical section with the actual final state. Long compute/encode work cannot be labeled non-interruptible merely for convenience.

# Backpressure/coalescing keys

Job kinds that can supersede work define a `CoalescingKey`, e.g.:

```
Thumbnail(ResourceId, SizeClass)
Histogram(PixelLayerId, AnalysisContext)
SceneEvaluation(DocumentId, ViewId)
FontScan(SourceRoot)
```

Queueing a newer equivalent request can cancel/replace an older queued computation. Coalescing never merges user-launched exports/saves whose outputs are independently expected.

# Memory admission

Memory-heavy jobs estimate peak working set and request admission before large allocations. Scheduler may queue/throttle them rather than letting multiple 2–4 GiB tasks start simultaneously. Actual memory is observed and quota violations become diagnostics/cancellation when possible.

# I/O admission

Limit concurrent heavy disk readers/writers per device/process so autosave, export and thumbnail decode do not saturate storage unpredictably. Foreground explicit Save receives priority over background asset indexing.

# GPU job policy

Compute/effect jobs submitted to GPU are tagged with generation/owner. GPU completion cannot publish stale output to newer cache state. Device loss cancels/discards device-generation-bound jobs and reschedules required work after renderer recovery.

# Document close

Close protocol queries JobSystem by document owner:

- discardable derived jobs → cancel;
- export jobs → may continue as snapshot-bound application/batch-owned jobs if user expects output;
- import/mutation jobs targeting the closing document → cancel/rollback before destruction;
- explicit Save → close waits for resolved save outcome according to UI flow;
- plugin jobs → follow plugin/document ownership and permission lifetime.

No job may later callback into a freed DocumentSession.

# Shutdown

Application shutdown has ordered phases: stop accepting nonessential jobs → resolve document saves/close → cancel background/plugin jobs → wait bounded grace period → force-drop only discardable work → flush diagnostics/recovery metadata → terminate executors. No detached worker is allowed to keep writing application state after shutdown boundary.

# Panic containment

Rust panics from ordinary worker closures are caught/converted where feasible at JobSystem boundary, but `unsafe`/process-corrupting faults are not assumed recoverable. A failed worker's partially constructed result is never published. Repeated subsystem panics can trip a circuit-breaker/disable path rather than infinite respawn.

# Progress contract

Progress event:

```
JobProgress {
  phase_id,
  completed_units?,
  total_units?,
  fraction?,
  current_item_semantic_id?,
  cancellable,
  warning_count
}
```

Only expose fraction when mathematically meaningful. Progress events are rate-limited/coalesced so thousands of tile events do not flood UI/MCP.

# Job persistence

Ordinary jobs are session state, not persisted. Recovery records enough metadata to restore document work, not arbitrary executor continuations. **Resumable cross-session export/download jobs are POST_V1_CANDIDATE** and require a dedicated versioned job-persistence ADR before implementation.

# Observability

Developer Job inspector shows queue by priority/domain, wait/run duration, owner, input revision, memory estimate/actual where available, cancellation state, stale-discard count and worker domain. Tracing spans connect parent Action/Command/Plugin/MCP invocation.

# Required stress gauntlets

- continuous brush input while 1000 thumbnails queue;
- Save during active background indexing and histogram;
- several memory-heavy exports with admission control;
- document close while every job state is exercised;
- plugin disable with child job tree;
- cancellation requested during noninterruptible atomic-replace phase;
- GPU loss with pending compute;
- randomized job completion order produces same committed document;
- no callback/use-after-owner after document/window/module teardown;
- starvation test proving low/background eventually runs while interactive latency remains within budget.
# 09.4 — Evaluation Graph, Derived Data, Invalidation & Cache Architecture

# Purpose

Non-destructive editing means visible output is evaluated from canonical sources. Aubrieta needs dependency-aware incremental evaluation rather than global rebuilds.

# Derived domains

Bounds, flattened paths, boolean results, stroke outlines, text layout, effect output, raster mip/tile state, thumbnails, spatial indexes, scene fragments and export-preflight facts are derived.

# Dependency graph

Each derived node records inputs by stable IDs/revisions and dependency kind. Mutation emits invalidation classes such as Geometry, Transform, Paint, TextContent, Typography, RasterPixels, EffectParameters, ResourceContent, ColorContext, Hierarchy.

# Granularity

Changing fill color must not invalidate geometry; moving an object invalidates world bounds/scene placement but not local path topology; editing one text span invalidates affected layout chain; changing an ICC proof profile invalidates display conversion but not canonical colors.

# Cache keys

Keys include object/resource identity, relevant revision/fingerprint, evaluator version and context (zoom/scale only when truly needed). Never key persistent truth by pointer address.

# Memoization policy

Caches have explicit ownership, memory weight, eviction priority and rebuild cost. GPU caches and thumbnails are discardable. Large-document memory pressure triggers deterministic eviction rather than uncontrolled growth.

# Scheduling

Invalidated work may evaluate lazily on demand or proactively in background based on priority. Visible canvas and input feedback outrank thumbnails/indexing.

# Cycle handling

Evaluation dependencies must be acyclic or use a documented fixed-point/feedback system. Plugin effects cannot create hidden dependency cycles.

# Diagnostics

Developer mode can inspect dependency edges, dirty reasons, cache hits/misses, rebuild duration and memory weight.

# Tests

Mutation-specific invalidation tests, cache poisoning prevention, random edit sequences, cold/warm equivalence, eviction/rebuild equivalence and large dependency-chain stress.

# Evaluation node contract

A derived computation is modeled conceptually as:

```
EvaluationKey {
  domain/type,
  owner stable ID,
  local input fingerprint/revision,
  evaluator_version,
  contextual key (only when semantically required)
}
EvaluationRecord {
  dependencies: Vec<DependencyKey>,
  generation,
  dirty_reasons,
  status,
  memory_weight,
  last_used,
  result_handle | diagnostic
}
```

The implementation does not need one universal boxed DAG type, but every derived subsystem must expose equivalent dependency/fingerprint semantics so invalidation is explainable and testable.

# Revision/fingerprint hierarchy

Use the narrowest stable version token that preserves correctness:

- document committed revision for broad snapshot identity;
- object semantic revision;
- resource content fingerprint/revision;
- specialized sub-revisions such as geometry/appearance/text/layout/raster where profiling justifies them;
- evaluator version bumped when algorithm semantics change.

Never use wall-clock time or pointer identity as validity.

# Dirty propagation

`ChangeSet` supplies seed IDs + mutation classes. Each evaluator maps classes to affected outputs. Propagation rules are declared close to the evaluator and unit-tested.

Examples:

- `Transform` → world bounds, spatial index placement, render placement; no local path flatten cache invalidation;
- `Geometry` → local bounds, stroke outline, boolean dependents, snap geometry, render vector fragment;
- `Paint` → appearance/render fragment, not path topology;
- `TextContent` → shaping/layout chain + bounds + scene glyph runs;
- `ColorContext` proof/display change → display transforms/render output, not canonical swatch/object values;
- `ResourceContent(Image)` → dependent image fragments/thumbnails/preflight, not unrelated object geometry.

# Stale-result rule

Background evaluation always captures an input fingerprint/generation. Before publishing a result, the scheduler compares the captured key against current required key. A stale result is discarded without mutating caches visible as current truth. Stale completion is normal control flow, not an error.

Expensive jobs should use cancellation cooperatively, but **generation checking is still mandatory** because cancellation can race or be unsupported inside third-party code.

# Cache state machine

```
Missing → Queued → Computing → Ready
                   ↘ Failed
Ready --invalidate--> Dirty/Queued
Any non-canonical state --evict--> Missing
```

A `Failed` derived result carries structured diagnostic and retry policy. Failure never becomes canonical document corruption; the editor may render a fallback/placeholder while preserving source data.

# Cache classes

At minimum distinguish:

1. **interactive-hot** — visible bounds, active text layout, current scene fragments;
2. **rebuildable-medium** — spatial indexes, flattened paths, stroke outlines;
3. **preview/background** — thumbnails, export previews, histograms;
4. **GPU residency** — device-generation bound;
5. **disk-backed derived** — optional caches that can be dropped across versions.

Eviction policy considers memory weight + rebuild cost + visibility/recency + class priority, not LRU alone.

# Memory pressure contract

Aubrieta diagnostics reports cache memory by class/subsystem. Under pressure, eviction order is deterministic enough for reproducible tests: offscreen/background previews before hot interaction caches, derived CPU/GPU copies before canonical raster/document storage. A cache eviction must never discard the only canonical pixel/source data.

# Dependency-cycle policy

Core evaluators must form a DAG. Registration validates declared plugin/custom evaluator dependencies when possible. Dynamic cycle detection during evaluation returns a diagnostic containing the dependency path. V1 does not implement general fixed-point evaluation; any feature requiring feedback must receive a dedicated ADR and explicit convergence contract.

# Snapshot consistency

One evaluation request uses a consistent committed document revision plus declared preview transaction state where explicitly requested. A single output must not accidentally combine geometry from revision N with style from revision N+1. Read snapshots/immutable views are preferred for background jobs.

# Scene/export consumers

Render scene, export and preflight may request evaluated data through the same evaluator contracts but with different contexts/quality levels. Export never consumes low-quality interactive approximations unless the exporter explicitly requests that representation.

# Quality levels

Evaluators that support preview/final quality expose named quality modes, not magic tolerances: e.g. `InteractivePreview`, `Display`, `Export`. The mapping to tolerances/resolution is centralized and testable. Final committed semantic operations such as Bake/Expand use deterministic final-quality parameters recorded where needed.

# Required observability

Developer mode must expose for each inspected object/evaluator:

- current key/generation;
- dirty reasons;
- dependency/dependent count;
- queued/computing/ready/failed state;
- last duration;
- memory weight;
- hit/miss/recompute counts;
- stale-result discard count.

This inspector is diagnostic derived data and never a dependency of production behavior.
# 01.4 — Event, ChangeSet, Invalidation & Notification Contract

# ChangeSet

Result of transaction describes:

revision before/after;

created/deleted/changed ObjectIds;

hierarchy changes;

PropertyIds changed;

resource/style/symbol changes;

coarse invalidation hints;

transaction/action/source metadata.

# Purpose

ChangeSet is not an event bus command. Mutation already happened atomically. Consumers use it to update projections/caches.

# Consumers

History, derived-data invalidation, renderer scene builder, Qt models, preflight/histogram schedulers, resource watchers and audit/diagnostics.

# Delivery

Session publishes ordered ChangeSets after commit outside document write locks. Slow consumers cannot block canonical mutation indefinitely; UI can coalesce projection updates.

# Granularity

Enough to avoid full rebuild, but no requirement to expose every internal mutation. Property/hierarchy/resource categories standardized.

# Derived dependencies

Dependency graph maps canonical fields to derived nodes. Property fill color invalidates render/thumbnail, not geometry boolean; path nodes invalidate geometry/bounds/snap/render.

# Tests

Order, no callback under write lock, coalescing, deleted object notification and correct cache invalidation.
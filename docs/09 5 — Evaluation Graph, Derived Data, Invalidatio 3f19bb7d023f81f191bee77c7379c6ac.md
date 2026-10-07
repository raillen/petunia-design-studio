# 09.5 — Evaluation Graph, Derived Data, Invalidation & Cache Architecture

# Derived principle

Anything reproducible from canonical state is disposable.

# Derived domains

Bounds, spatial index, path tessellation, boolean evaluated geometry, text layout, image mipmaps, effect intermediate surfaces, thumbnails, histogram, render scene, export staging.

# Evaluation graph

Nodes identified by semantic input IDs + revision/parameter fingerprints. Dependencies form DAG; cycles rejected for effects/resources that require acyclic evaluation.

# Invalidation

ChangeSet maps property/structure changes to minimal invalidation scopes. Changing fill color does not recompute path boolean geometry; path node mutation invalidates bounds/tessellation/snap/render.

# Caches

Memory budgets per class; LRU/clock policy; priority interactive > visible > near viewport > background. Cache key includes document revision components/profile/quality where semantics require.

# Threading

Derived computation can run workers if input snapshot immutable. Publish result only if generation/revision still relevant; stale result discarded.

# GPU cache

Textures/buffers are render-backend resources and never serialized as truth. Device loss drops/rebuilds them.

# Diagnostics

Dev panel exposes hit/miss, bytes, invalidations, stale jobs and expensive nodes.
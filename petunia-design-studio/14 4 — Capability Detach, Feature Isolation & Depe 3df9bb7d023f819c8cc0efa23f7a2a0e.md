# 14.4 — Capability Detach, Feature Isolation & Dependency Conformance

<aside>
🧩

Modularity is proven by removal, not by crate names. Built-in capabilities are tested as if they were detachable extensions.

</aside>

# Detach matrix

Every optional capability records:

- provider CapabilityId;
- required capabilities;
- allowed dependency direction;
- persistence/resources it owns;
- behavior when absent;
- runtime enable/disable/unload support;
- compile-time feature/build support where relevant;
- DetachTestIds.

# Required examples

Raster Engine removed → Design vector/layout still opens and operates.

Photo Persona removed → document model and Design remain intact.

Data Merge removed → ordinary document editing unaffected.

MCP removed → desktop/headless core still works.

Lua runtime removed → application works without embedded scripting.

PDF exporter removed → other exporters remain.

Optional plugin/resource pack absent → safe fallback and diagnostic.

# Compile graph proof

CI rejects forbidden dependency edges and cycles. Domain/application crates cannot acquire GPUI or adapter-specific dependencies through convenience helpers.

# Runtime proof

When a module is runtime detachable, tests cover registration, use, cancellation, disable, unload, resource cleanup and re-enable where supported.

# Missing-provider behavior

Capability requests fail closed with structured disabled reasons. Neighboring UI hides or disables affordances semantically; no null-pointer style assumptions.

# Persistence

Documents/configurations referencing unavailable optional capabilities preserve unknown/unsupported data according to the format contract rather than silently deleting it.

# Collision tests

Duplicate semantic IDs, conflicting providers and invalid registration order fail deterministically with diagnostics.

# Randomized composition

Where deterministic, CI may vary optional capability enablement/order to expose hidden side dependencies.

# Release gate

A newly optional subsystem is not modular merely because its crate exists separately; the detach matrix and at least one proof configuration are mandatory.
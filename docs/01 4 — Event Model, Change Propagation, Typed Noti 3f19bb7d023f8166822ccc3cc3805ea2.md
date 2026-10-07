# 01.4 — Event Model, Change Propagation, Typed Notifications & Feedback-Loop Prevention

# Event classes

DocumentCommitted(ChangeSet), SelectionChanged, ViewportChanged, ActiveToolChanged, WorkspaceChanged, JobChanged, ResourceStatusChanged, ThemeChanged, PluginContributionChanged.

# Rule

Events report completed state changes. They do not replace Commands for document mutation.

# Delivery

C++ application event queue to Python/main-thread dispatcher. Batches/coalescing for high-frequency view/job events. Event includes source/revision/generation.

# Subscriptions

Scoped RAII/native token or Python lifecycle subscription; unsubscribe on owner destruction. Avoid global anonymous lambdas that retain panels forever.

# Feedback loops

Panel edits start Command; DocumentCommitted returns new snapshot; panel updates control without emitting another edit event. BindingController differentiates programmatic model update from user change.

# Ordering

Within one transaction: canonical commit -> ChangeSet/revision -> derived invalidation schedule -> application event -> UI refresh. Selection changes caused by deleted objects happen deterministically after commit and can be bundled.

# Errors

Event handler exception cannot roll back already committed document; top-level UI boundary logs/isolates failure and model can resync.

# Tests

Two panels editing same property, stale event, destroyed panel, coalesced job progress and commit-selection ordering.
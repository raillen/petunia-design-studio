# 19.25 — Job Center / Background Tasks Panel

# Identity

PanelId [ptnd.panel.jobs](http://ptnd.panel.jobs), optionally popover from status bar.

# Model

Active/recent JobSnapshot: type, owner doc, phase, progress, started time, cancellable, status, result/report link.

# Grouping

Jobs grouped by document/type when many small tasks exist; internal ephemeral thumbnail jobs generally hidden unless dev mode.

# Actions

Cancel, Retry eligible failed, Open Report, Reveal Output, Clear completed. Pause only if job type explicitly supports.

# Progress

Determinate current/total or indeterminate phase. Updates throttled; ETA only if statistically meaningful.

# Errors

Structured failure summary + details/copy diagnostic. Retry verifies source revision/resources.

# Accessibility

Progress role/status announcements not spammy; cancellation keyboard accessible.

# Tests

Many jobs, cancellation, app close, retry, stale export snapshot and plugin job crash.
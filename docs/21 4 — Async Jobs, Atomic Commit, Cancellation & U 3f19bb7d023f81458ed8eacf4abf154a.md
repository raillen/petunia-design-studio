# 21.4 — Async Jobs, Atomic Commit, Cancellation & Undo for Expensive Operations

# Pattern

Long operation must not hold writable document lock:

snapshot inputs -> native/background compute -> staged result -> validate current revision/conflict policy -> short atomic commit transaction.

# Revision

Job stores input revision and affected dependencies. At completion:

- if unchanged, commit;
- if unrelated changes allowed, rebase only when operation defines safe merge;
- otherwise return stale result and offer rerun.

# Cancel

Before commit, cancellation discards staged result. During short commit cancellation is not observed halfway; transaction completes atomically or rolls back.

# Examples

Inpaint, large boolean, filter-to-pixels, image resize, batch generated resources.

# History

Only successful commit creates history entry. Computation progress itself never appears in History.

# Job ownership

Closing view may leave session job according policy; closing last session asks/cancels if job would produce document mutation.

# Tests

Cancel at every stage, stale revision, compute failure, commit validation failure, app shutdown and undo immediately after async commit.
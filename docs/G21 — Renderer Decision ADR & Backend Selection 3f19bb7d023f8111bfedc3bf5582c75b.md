# G21 — Renderer Decision ADR & Backend Selection

# Goal

Select the primary renderer backend from evidence, not preference.

# Depends

G20.

# Authority

04.6, 15.3, 09.8.*, 25.

# Owner

technology-decision-agent; architect owns ADR; renderer-engineer supplies implementation evidence.

# Deliverables

Pareto matrix, raw benchmark links, visual correctness comparison, platform/driver matrix, maintenance/license analysis and ADR locking primary/fallback strategy.

# Acceptance

Winner satisfies hard requirements on Windows/Linux/macOS, offscreen/headless, device loss, color hooks and acceptable memory/perf. Rejected candidates have explicit reasons.

# Non-goals

No large production renderer implementation before decision.

# Evidence

Signed-off ADR and benchmark bundle.
# G021 — Renderer Technology ADR & Production Backend Selection

# Goal

Select production renderer backend from evidence, not preference.

# Depends

G020.

# Primary

technology-decision-agent + architect. Verification: performance-agent, renderer-engineer.

# Skills

benchmarking, performance-native, architecture-quality, rendering-2d, grounded-implementation.

# Deliverables

Candidate elimination matrix; hard-constraint compliance; benchmark comparison; integration/maintenance/security/license analysis; selected backend ADR; rejected alternatives and revisit triggers; production backend migration plan; fallback/reference strategy.

# Acceptance

ADR identifies one production path and proves it meets target platforms/offscreen/device-loss/color-hook requirements. Decision leaves RenderScene/domain backend-neutral.

# Evidence

Raw benchmark bundle, visual diffs, prototype commits and platform notes.

# Non-goals

Full renderer implementation or polishing benchmark adapters.
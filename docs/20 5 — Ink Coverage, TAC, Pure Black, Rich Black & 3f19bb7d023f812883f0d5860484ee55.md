# 20.5 — Ink Coverage, TAC, Pure Black, Rich Black & Black Preservation

# TAC

Total Area Coverage = C+M+Y+K at proof/output pixel after conversion, excluding spot unless separate policy. Threshold comes from output preset/profile/printer spec; no universal hidden value.

# Heatmap

Ink Coverage view computes threshold exceedance and displays heatmap/overlay. Info panel shows exact process percentages at pointer.

# Preflight

Rules flag areas above TAC with approximate bounds/Surface and max observed value. Sampling resolution strategy balances accuracy/performance; Final Preflight uses authoritative setting.

# Pure black

Define pure K as C=M=Y=0,K>0 within tolerance in canonical/output space. “Preserve pure black” conversion/export option keeps eligible text/vector K-only where ICC workflow allows.

# Rich black

Petunia never automatically changes user process black into rich black without explicit command/preset. Suggested recipes can be user/printer presets.

# Small text

Preflight can warn small text using multi-ink rich black if registration risk rule enabled.

# Tests

TAC ramps, profile conversion, K-only text, rich-black solids, spot colors and heatmap consistency.
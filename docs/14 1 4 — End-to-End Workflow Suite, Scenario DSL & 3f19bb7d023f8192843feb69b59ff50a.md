# 14.1.4 — End-to-End Workflow Suite, Scenario DSL & Deterministic Replay

# Scenarios

Human-readable scenario manifest names fixture, initial workspace, semantic actions/tool gestures, checkpoints and final assertions.

# Layers

Prefer semantic Actions for product behavior; UI interaction path additionally tests actual widgets/tool gestures. Both should converge to same canonical outcome where equivalent.

# Checkpoints

Selection/tool/context state, document revision, history depth, semantic snapshot, visible panels, export report and expected diagnostic warnings.

# Replay

Scenario can run headless core, Python application facade, Qt semantic UI and MCP subset depending tags.

# Failure artifact

Minimal logs, semantic tree snapshot, canonical diff, screenshot/visual diff and last Actions/Commands.

# Critical scenarios

First vector workflow, mixed photo poster, Data Merge batch, recovery, plugin permission denial/crash and MCP export.

# Stability

No sleeps for correctness; waits are condition/job/revision based with timeout only as failure bound.
# 08.29 — Import, Place, Export, Preflight & Workflow Completion UX

# End-to-end principle

A workflow is not complete at “Export button exists”. It must cover source selection, interpretation, missing resources, fidelity, destination conflict, progress, validation and report.

# Import

Only ask questions with real semantic impact. Defaults recommended, choices remembered by format when safe.

# Place

Staged placement makes linked/embedded state visible; Resource Manager offers later conversion.

# Preflight

Issues navigate to affected ObjectId/Surface, explain risk and offer fix action only when deterministic. Filters by severity/type.

# Export

Format capabilities dynamically control settings. Disabled unsupported setting explains why. Preflight summary never buried behind advanced tab.

# Completion

Success notification links Reveal/Open Folder/Copy Path/Open Report. Batch result distinguishes succeeded/skipped/failed.

# Retry

Failed outputs can retry individually without regenerating successful outputs unless source revision changed.

# Automation

Headless/MCP workflows receive same structured ImportReport/PreflightReport/ExportReport with no GUI-only information required.
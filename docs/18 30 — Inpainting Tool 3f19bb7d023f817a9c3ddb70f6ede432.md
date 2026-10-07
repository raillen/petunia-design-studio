# 18.30 — Inpainting Tool

# Identity

ToolId ptnd.tool.inpaint.

# Interaction

User paints region mask to remove/replace. Brush mask is staged and visualized distinctly from result.

# Engine

IInpaintEngine receives immutable source snapshot, region mask, optional context radius/quality and deterministic seed. Built-in classical algorithm is local/offline baseline; external/AI providers require explicit plugin/network permissions.

# Job

Pointer stroke can accumulate mask immediately; Apply/automatic-on-release starts cancellable native job. Job result is StagedRasterPatch tied to source revision.

# Commit

Before commit, validate target/resource revision. If incompatible edit occurred, offer rerun/reject rather than blindly overwrite. Commit swaps affected tiles atomically.

# Context

brush size/hardness, source scope if applicable, quality, clear mask, preview/apply, provider when multiple engines installed.

# Privacy

No artwork leaves machine unless user explicitly selected a network-capable provider and granted scope.

# Tests

Large regions, edge borders, cancellation, stale revision, deterministic local result, plugin provider failure and undo.
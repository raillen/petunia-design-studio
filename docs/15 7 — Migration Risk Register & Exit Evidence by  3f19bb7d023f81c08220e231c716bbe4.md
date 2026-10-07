# 15.7 — Migration Risk Register & Exit Evidence by Subsystem

# Risk categories

Data loss, visual fidelity, performance regression, platform regression, accessibility loss, plugin/API incompatibility, build/deployment complexity and undocumented old behavior.

# Subsystems

Document/commands, PTND, vector geometry, raster, text, color, renderer, UI/workspace, plugins, MCP and import/export each maintain:

risk;

probability/impact;

mitigation;

fixture;

owner;

exit evidence.

# High risk

PTND writer/reader, text shaping/layout, color/prepress and renderer blending require independent reference evidence before old authority retired.

# Unknown behavior

Explorer records code-observed legacy behavior not in docs. Architect/product decides preserve/fix/drop with ADR/status.

# Exit

No subsystem marked Migrated solely because code compiles; must pass semantic/functional evidence and documentation update.
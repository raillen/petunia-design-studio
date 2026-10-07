# 07.5 — Plugin/MCP Semantic Parity & No-Backdoor Rule

# Rule

If GUI mutation has an Action/Command, plugins/MCP invoke the same semantic path. No direct document container writes, hidden debug endpoint or raw widget mutation.

# Parity

Equivalent UI and API operation should produce equal canonical snapshot/change set given same input context.

# Exceptions

Pure UI view state may use UI-specific service; developer synthetic input is testing capability, not document API.

# Permissions

Semantic parity does not imply equal authority: plugin/MCP still pass permission checks and file/network brokers.

# Discoverability

Action/Property schema registry powers command palette, plugin SDK and MCP descriptions to reduce drift.

# Tests

Golden workflows execute direct Action, plugin request and MCP request; compare snapshot/revision/history labels.
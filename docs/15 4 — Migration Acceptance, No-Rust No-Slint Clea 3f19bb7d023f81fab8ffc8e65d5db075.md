# 15.4 — Migration Acceptance, No-Rust/No-Slint Cleanup & Parity Exit Criteria

# Exit criteria

- current notebook contains no active Rust/Slint authority;
- repository production dependencies contain no unintended Rust/Slint path;
- all V1 old requirements map in parity ledger;
- PTND fixtures open/save/migrate;
- UI tool/panel coverage matches V1 ledger;
- plugin/MCP contracts implemented to declared milestone;
- Python strict typing clean;
- C++ sanitizer gates clean;
- release artifacts pass platform smoke.

# Allowed legacy

Historical docs, migration readers, fixtures and archived branches may mention old stack explicitly.

# Search gate

CI/release checklist searches current docs/config/source for forbidden obsolete identifiers with allowlist for migration/history.

# Behavioral parity

Port is accepted by semantic/UX evidence, not by matching line count or internal class names.

# Regression handling

If old implementation had behavior undocumented but valuable, capture it as explicit requirement before porting rather than silently copying accidental behavior.
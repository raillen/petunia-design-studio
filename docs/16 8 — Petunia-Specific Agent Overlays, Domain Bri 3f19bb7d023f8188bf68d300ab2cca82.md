# 16.8 — Petunia-Specific Agent Overlays, Domain Briefs & Microcontext Packages

# Overlay

tooling/prumo/petunia-overrides contains concise additions such as canonical notebook links, repository boundaries, forbidden dependency edges, build/test entry points and domain-specific invariants.

# Domain briefs

core-document, qt-ui, geometry, raster, renderer, text, color, ptnd-io, plugins, mcp and release. Each brief points to authoritative pages and current public headers/schemas.

# Microcontext package

Generated per Task: Goal criteria, domain brief, relevant ADRs, exact interfaces/schemas, nearby implementation and tests. Exclude unrelated notebook bulk.

# Staleness

Microcontext records source revision/doc version; regenerate after upstream task/contract changes.

# Security

Never include secrets/private user documents in generated agent context. Test fixtures and redacted diagnostics only.

# Maintenance

Documentation-maintainer owns domain-brief drift checks; project code remains authority for implemented signatures while notebook remains semantic authority.
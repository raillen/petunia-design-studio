# 16.4 — Workforce Vendoring, Pinning, Updates & Project Directory Contract

# Goal

Petunia must have local, reproducible access to the exact agent/skill/recipe instructions used by code agents.

# Suggested directory

```
tooling/prumo/
  UPSTREAM.md
  commit.txt
  agents/
  skills/
  recipes/
  manifests/
```

# Pin

Record upstream repository, commit SHA, date and selected paths. Do not pull mutable main during deterministic implementation run.

# Selection

Vendor required agents from 16.1, required skills from 16.2 plus manifest-declared dependencies, and recipes from 16.3. Extra skills may be added per Goal.

# Update workflow

1. fetch new upstream;
2. diff selected workforce content;
3. review changed role permissions/invariants;
4. run compatibility/docs checks;
5. update pin;
6. record changelog.

# No silent overwrite

Local Petunia-specific overlays should not edit vendored upstream files directly. Put overlays/instructions in tooling/prumo/petunia-overrides and compose them so upstream diff remains clear.

# Agent router

Project docs map task categories to primary agent/recipe/skills. Agents still read manifests; router is convenience, not replacement.

# CI

Verify required paths exist, commit pin matches vendored manifest metadata if script supports it, no missing transitive skills and no accidental local edits to vendored snapshot.

# Licensing

Preserve upstream licensing/attribution requirements and include them in SBOM/release docs if bundled in distributed source/tools.

# Current upstream snapshot observed

As of 2026-10-06, Prumo main resolves to commit **e213260c99d22c89bf31890ec595a89725031c71** (commit dated 2026-10-02). When the Petunia repository actually vendors workforce files, pin the exact commit used at that moment and do not assume this notebook observation remains latest.
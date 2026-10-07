# 16.7 — Workforce Dependency Resolution, Mandatory Skill Closure & Validation

# Problem

Agent manifests can require skills; recipes can reference agents/skills. Vendored subset must include transitive closure.

# Resolver

Tooling reads selected agent/recipe manifests, discovers mandatory skill dependencies and validates local vendored paths. Missing dependency fails workforce validation.

# Pinning

Every vendored file comes from one recorded upstream commit unless explicit local override. Mixed upstream commits require manifest and reason.

# Overrides

Petunia-specific instructions live separate overlay and can tighten but must not silently contradict upstream safety/role rules without reviewed project ADR.

# Validation

CI checks file existence, upstream commit metadata, no direct edits to vendored files, recipe references resolved and task router names valid agents/skills.

# Upgrade

Diff manifests first because role responsibility/required skills may change even when implementation code does not.
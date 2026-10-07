# 15.7 — Incremental Cutover, Feature Flags, Parallel Oracles, Rollback & Branch Strategy

# Development

Migration occurs by vertical slices, not one big rewrite. New Python/C++ path becomes authoritative per subsystem only after gate.

# Parallel oracle

For geometry/render/save behaviors where legacy implementation remains runnable, deterministic fixtures can compare outputs. Oracle is temporary and removed after confidence.

# Feature flags

Internal migration flags may switch old/new subsystem during development, but release must not carry indefinite two-truth architecture unless explicit fallback product requirement.

# Branch

Main remains buildable. Large migration Goals use short-lived integration branches/worktrees with frequent semantic checkpoints, not months-long fork.

# Rollback

Before destructive removal, preserve known-good tag/branch and document data compatibility. If new release rollback cannot read newer PTND, release notes/update UI make this explicit.

# Cutover gate

Functionality, save/reopen, undo, performance, accessibility, platform and format evidence must pass for subsystem before legacy deletion.

# Cleanup

After cutover, remove adapters/feature flags/dependency/build code and update docs so agents cannot accidentally target legacy path.
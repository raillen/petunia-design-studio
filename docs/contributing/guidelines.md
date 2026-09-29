# Contribution guidelines

How to open PRs and meet the code standards. Short version: small diffs, green gates, docs in the same change.

## 1. Before you code

- Build the microcontext: goal, scope, exact page/ADR IDs, crates, capabilities, invariants. Never dump credentials or artwork into prompts or packets.
- Check scope status. A bare "later/planned" authorizes nothing — confirm `V1 Required` / `Milestone Required` / `Post-V1 Candidate` / `Research` / `Open ADR`.
- For UI work: no fake UI. A button without an action, `todo!()`, empty callback or placeholder is forbidden — implement it, disable it with a reason, hide it, or mark it experimental.

## 2. Code standards

- `cargo fmt --all` before every push; `cargo clippy --workspace --all-targets -- -D warnings` must be silent.
- Domain crates never import GUI toolkit types (`cargo run -p xtask -- architecture` enforces this).
- Every mutation flows UI/Shortcut/Plugin/MCP → Action → Command → DocumentMutator → ChangeSet.
- Typed stable IDs only (`ObjectId`, `SurfaceId`, …); never Vec indices, pointers or handles as identity.
- Non-destructive by default: Bake / Expand / Rasterize / Convert-to-Curves are explicit user operations.

## 3. Opening a PR

```bash
# Run the gates locally before pushing
cargo run -p xtask -- verify
```

1. One concern per PR; keep diffs reviewable (< ~400 lines preferred).
2. Include: focused tests + gauntlet evidence, headless-first proof, detach proof where applicable.
3. Update the Atlas, ADR registry and generated references **in the same change** that alters a contract.
4. Update **both** EN and pt-BR docs in the same change (see [Translations](/contributing/translations)). CI fails on drift.

## 4. Review expectations

- Reviewers check architectural conformance, security impact and constructive feedback — not style (fmt/clippy own that).
- Conflicts between agent guidance and the canonical atlases/ADRs resolve in favor of the atlases/ADRs; record the conflict in the PR.

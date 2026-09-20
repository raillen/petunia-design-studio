# 09.20 — Build, Packaging, Release, Updates, Dependency Policy, SBOM & Licensing

# Build profiles

Define dev, test/gauntlet, profiling, release and minimal/headless feature sets. Developer-only UI/inspection/profiling cannot silently ship enabled in production unless explicitly intended.

# Cargo workspace

Feature flags represent optional adapters/capabilities, not arbitrary behavior forks. Avoid combinatorial feature explosion; CI tests supported profiles.

# Reproducibility

Pin toolchain/dependency policies, commit lockfile for application, record rustc/Cargo version and build metadata. Deterministic resource-pack compilation and stable generated IDs.

# Packaging

Native packages/installers for Windows/macOS/Linux chosen per release plan. Bundle required runtime assets/profiles/resources explicitly; no dependence on developer machine paths.

# Signing/notarization

Windows signing and macOS signing/notarization become release gates when distributing binaries. Linux packages/checksums follow target ecosystem.

# Updates

Updater is a separate trusted subsystem. Signed metadata/artifacts, channel (stable/beta/nightly), staged download, checksum/signature validation, rollback/failure recovery. Never let a theme/plugin update use application-update privileges.

# Plugins/resource packs

Have their own compatibility/install/update metadata and storage roots. Application update does not delete user packs/plugins.

# Licenses

GPLv3 project obligations tracked; third-party license inventory and notices generated from audited manifest plus manually reviewed exceptions/assets/fonts/icons.

# SBOM/security

Generate SBOM for releases, dependency vulnerability audit and license policy. Native/FFI dependencies highlighted.

# Release gates

Clean build, tests/fuzz smoke, UI gauntlet subset, migration fixtures, export fixtures, package install/uninstall smoke, update/rollback rehearsal, checksum/signature verification and reproducible version/about metadata.

# Documentation build and publication gate

The release pipeline also owns the living VitePress documentation artifact defined in section 12.

For any release containing behavior/API/schema/UI changes, release gates include:

- canonical English repository docs updated;
- pt-BR translation synchronized;
- VitePress production build succeeds;
- broken links/anchors fail CI;
- generated Action/Property/Capability/Plugin/MCP/diagnostic/format references match source;
- tested documentation examples pass;
- documentation site accessibility smoke test passes;
- version/`next` banners and migration notes match the release channel;
- static site deployment is tied to the exact release/commit.

Documentation deployment may target GitHub Pages or another static host; essential reference remains readable from repository Markdown/static output without a proprietary runtime service.

# Plugin runtime packaging gate

The accepted **Lua 5.5 + `mlua` V1 scripting runtime** must pass installer/package smoke tests on all supported desktop platforms with **no developer-machine Lua dependency**; the runtime is vendored/bundled by Aubrieta. Native Lua/C module loading is disabled for ordinary plugins unless a separate trusted-extension ADR explicitly permits it. Binary/runtime size, startup cost, update compatibility, Lua/mlua version pinning and license/SBOM entries are release-gated. The optional Wasmtime/WASI high-isolation tier has its own feature/package profile and may not become an accidental mandatory dependency of minimal/headless builds unless the release profile includes that tier.
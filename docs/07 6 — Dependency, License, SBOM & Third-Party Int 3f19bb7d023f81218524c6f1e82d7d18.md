# 07.6 — Dependency, License, SBOM & Third-Party Intake Policy

# Intake

Every dependency has purpose, owner, version source, license, security exposure, update policy, transitive footprint and replacement risk.

# Native libraries

Especially scrutinize parsers/codecs/GPU/font/color libraries because malformed input reaches native code.

# License

Redistribution obligations, LGPL/commercial considerations, patents/codecs and asset/font licenses are recorded before release use.

# SBOM

Release produces machine-readable inventory for Python packages, native libraries, bundled runtimes/assets and relevant tooling.

# Updates

Security fixes prioritized. Major upgrades require compatibility/performance corpus. No floating unpinned dependency in release build.

# Removal

Unused dependencies are removed rather than kept as speculative infrastructure.
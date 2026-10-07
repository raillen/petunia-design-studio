# 07.6 — Dependency Risk, Licensing, Supply Chain & Update Governance

# Inventory

Every dependency records owner, purpose, license, version source, native attack surface, update cadence and replacement difficulty.

# Admission

New dependency requires reason it beats internal/simple solution, license compatibility, recent maintenance/security posture and binary/package impact.

# Pinning

Python lock; native package lock/commit; workforce commit; generated tool versions.

# Security

Vulnerability scanning is signal, not sole decision. Parser/codecs/fonts get fuzz corpus even when third-party upstream tests exist.

# Updates

Patch upgrades run focused regression. Major update with behavioral/API change requires ADR/compatibility review.

# SBOM

Release includes machine-readable SBOM and notices. Bundled assets/fonts/codecs included where appropriate.

# Removal

Abandoned/high-risk dependency has migration plan; adapter boundaries prevent canonical model lock-in.
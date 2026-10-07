# 22.8 — Resource Pack Security, Licensing, Import/Export, Backup & Recovery

# Package security

All packs use safe archive profile: no traversal/absolute paths/symlinks, bounded file count/size/compression ratio, strict schemas and content-type validation.

# Executability

Brush/style/asset/theme packs contain data only. Any code requires Plugin package and permissions.

# Licensing

Manifest may record author, source URL/identifier, license/SPDX and redistribution notes for embedded assets. Petunia does not infer permission to redistribute external fonts/images.

# Import staging

Parse/validate entire manifest/dependency graph before mutating library. Install into temp then atomic library update.

# Backup

User resource libraries can be exported as versioned backup bundle. Recovery never overwrites current library without preview/conflict plan.

# Corruption

Index can rebuild from individual resource manifests/packs where possible; corrupt resource isolated and reported.

# Sync conflict

Future cloud provider resolves by stable ID/version/fingerprint; core never “last write wins” blindly for user-created resource mutations.

# Tests

Malicious archives, interrupted import, duplicate IDs, corrupted index, backup restore and license metadata retention.
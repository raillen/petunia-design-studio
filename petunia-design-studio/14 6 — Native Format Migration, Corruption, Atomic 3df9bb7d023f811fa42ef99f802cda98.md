# 14.6 — Native Format Migration, Corruption, Atomic Save & Recovery Torture Suite

# Amendment 2026-09-21 — .PTND migration torture suite

The native-format torture suite now targets `.PTND`. Include legacy `.aubrieta/.aubri` fixtures only where real historical files exist. Required cases: atomic save, disk full, read-only, process interruption, temp-file residue, corrupt/truncated archive, unknown schema version, missing resource, legacy conversion, save-after-conversion, backup preservation and round-trip equivalence.

<aside>
💾

The native format is trustworthy only when old, interrupted, partially corrupt and forward-versioned documents behave predictably.

</aside>

# Migration graph

Every persisted schema version has explicit forward migration edges. CI tests direct and chained migrations for representative historical fixtures.

# Round trip

Open → migrate → save → reopen must preserve canonical semantics. Derived caches/compatibility projections may change without changing authoring truth.

# Corruption classes

Truncated ZIP/package, missing manifest, bad JSON/schema, duplicate IDs, dangling references, missing resources, invalid checksums, malformed compatibility projection, unsupported plugin data and oversized resources.

# Unknown data

Forward/extension data is preserved or rejected according to the documented compatibility policy. Unsupported fields must not be silently discarded when preservation is promised.

# Atomic save

Fault injection covers interruption before write, during resource write, before rename/replace, after temporary-file creation, disk full, permission loss and destination disappearance.

# Autosave/recovery

Recovery artifacts are distinguishable from explicit Save. Tests cover stale recovery, multiple recovery candidates, corrupted recovery and cleanup after successful save.

# Migration failure

Migration operates on staging state. Failure never partially mutates the original file or live canonical document. Diagnostic identifies version/edge and safe next action.

# Aliases

.aubrieta and .aubri are the same format identity. Tests prevent schema divergence by extension.

# Interoperability representations

Derived SVG/PDF projections inside the package cannot override richer canonical authoring data.

# Evidence bundle

Release includes migrated fixture summary, corrupt-input outcome matrix and atomic-save fault-injection status.